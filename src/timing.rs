use byteorder;
use byteorder::ByteOrder;
use chrono;
use chrono::Datelike;
use chrono::Duration;
use chrono::NaiveDateTime;
use chrono::Timelike;
use chrono::Utc;
use diesel;
use diesel::sqlite::SqliteConnection;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use models::NewNpcInstance;
use models::NpcClass;
use models::NpcInstance;
use rand::distributions::{Bernoulli, Distribution};
use rand::ChaChaRng;
use rand::SeedableRng;
use schema::{npc_classes, npc_instances};
use std::mem;

#[rustfmt::skip]
fn seed(dt: &NaiveDateTime, id: i32) -> [u8; 32] {
    let mut id_slice = [0, 0, 0, 0];
    byteorder::LittleEndian::write_i32(&mut id_slice, id);
    [
        dt.year() as u32 as u8,
        dt.month() as u8,
        dt.day() as u8,
        dt.hour() as u8,
        dt.minute() as u8,
        dt.second() as u8,
        id_slice[0],
        id_slice[1],
        id_slice[2],
        id_slice[3],
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0
    ]
}

pub fn fast_forward_instances(
    connection: &SqliteConnection,
    shift: chrono::Duration,
) -> Result<(), String> {
    let time = Utc::now().naive_utc();
    let forward_time: NaiveDateTime = time + shift;
    let tick = Duration::seconds(10);
    let classes = npc_classes::table
        .filter(npc_classes::dsl::active.gt(0))
        .load(connection)
        .map_err(|e| {
            format!(
                "could not query database for npc classes: {}",
                e.to_string()
            )
        })?;

    for mut class in classes {
        let instances = create_instances(&mut class, &forward_time, &tick, &Duration::minutes(20));

        diesel::insert_into(npc_instances::table)
            .values(&instances)
            .execute(connection)
            .map_err(|e| format!("could not insert npc instances: {}", e))?;

        diesel::update(npc_classes::table)
            .filter(npc_classes::dsl::id.eq(class.id))
            .set(npc_classes::dsl::next_tick.eq(class.next_tick - shift))
            .execute(connection)
            .map_err(|e| format!("could not update npc next generation: {}", e))?;
    }

    for mut instance in npc_instances::table
        .load::<NpcInstance>(connection)
        .map_err(|e| format!("could not collect npc instances: {}", e))?
    {
        diesel::update(npc_instances::table)
            .filter(npc_instances::dsl::id.eq(instance.id))
            .set(npc_instances::dsl::active_until.eq(instance.active_until - shift))
            .execute(connection)
            .map_err(|e| format!("could not shift npc expiration date: {}", e))?;
    }

    diesel::delete(npc_instances::table)
        .filter(npc_instances::dsl::active_until.lt(time))
        .execute(connection)
        .map_err(|e| format!("could not remove old npc instances: {}", e))?;

    Ok(())
}

pub fn update_instances(connection: &SqliteConnection) -> Result<(), String> {
    let time: NaiveDateTime = Utc::now().naive_utc();
    let tick = Duration::seconds(10);
    let classes = npc_classes::table
        .filter(npc_classes::dsl::active.gt(0))
        .load(connection)
        .map_err(|e| {
            format!(
                "could not query database for npc classes: {}",
                e.to_string()
            )
        })?;

    for mut class in classes {
        let instances = create_instances(&mut class, &time, &tick, &Duration::minutes(20));

        diesel::insert_into(npc_instances::table)
            .values(&instances)
            .execute(connection)
            .map_err(|e| format!("could not insert npc instances: {}", e))?;

        diesel::update(npc_classes::table)
            .filter(npc_classes::dsl::id.eq(class.id))
            .set(npc_classes::dsl::next_tick.eq(class.next_tick))
            .execute(connection)
            .map_err(|e| format!("could not update npc next generation: {}", e))?;
    }

    diesel::delete(npc_instances::table)
        .filter(npc_instances::dsl::active_until.lt(time))
        .execute(connection)
        .map_err(|e| format!("could not remove old npc instances: {}", e))?;

    let unique: Vec<NpcClass> = npc_classes::table
        .filter(npc_classes::dsl::unique.eq(1))
        .load(connection)
        .map_err(|e| {
            format!(
                "could not query database for unique classes: {}",
                e.to_string()
            )
        })?;

    for class in unique {
        if let Ok(first) = npc_instances::table
            .filter(npc_instances::dsl::class.eq(class.id))
            .order((npc_instances::dsl::active_until.desc(), npc_instances::dsl::id.desc()))
            .first::<NpcInstance>(connection) {
                diesel::delete(npc_instances::table)
                    .filter(npc_instances::dsl::class.eq(class.id))
                    .filter(npc_instances::dsl::id.ne(first.id))
                    .execute(connection)
                    .map_err(|e| format!("could not remove older unique npc instances: {}", e))?;
        }
    }

    Ok(())
}

pub fn create_instances(
    class: &mut NpcClass,
    final_time: &NaiveDateTime,
    tick: &chrono::Duration,
    visit_time: &chrono::Duration,
) -> Vec<NewNpcInstance> {
    let mut result = Vec::new();
    let probability = ((tick.num_seconds() as f64) / (visit_time.num_seconds() as f64))
        * ((class.commonality as f64) / 100f64);
    let distribution = Bernoulli::new(probability);
    let mut last_tick = None;

    while &class.next_tick <= final_time {
        let mut rng = ChaChaRng::from_seed(seed(&class.next_tick, class.id));
        (0..10).for_each(|_| {
            distribution.sample(&mut rng);
        });
        if distribution.sample(&mut rng) {
            result.push(NewNpcInstance {
                class: class.id,
                active_until: class.next_tick + *visit_time,
            });
        }
        let next_tick = class.next_tick + *tick;
        last_tick = Some(mem::replace(&mut class.next_tick, next_tick));
    }

    if let Some(last_tick) = last_tick {
        result.retain(|v| v.active_until >= last_tick);
    }
    result
}
