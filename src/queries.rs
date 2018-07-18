use chrono::Utc;
use diesel;
use diesel::sqlite::SqliteConnection;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use models::Alias;
use models::Constant;
use models::NewAlias;
use models::NewNpcClass;
use models::NpcClass;
use models::NpcInstance;
use schema::{aliases, constants, npc_classes, npc_instances};

pub fn add_class(
    connection: &SqliteConnection,
    name: String,
    commonality: i32,
    active: bool,
) -> Result<(), String> {
    let time = Utc::now().naive_utc();
    let class = NewNpcClass {
        name: &name,
        commonality,
        next_tick: time,
        active: if active { 1 } else { 0 },
    };
    diesel::insert_into(npc_classes::table)
        .values(&class)
        .execute(connection)
        .map_err(|e| format!("could not insert new npc: {}", e.to_string()))?;
    Ok(())
}

pub fn change_active(
    connection: &SqliteConnection,
    name: String,
    active: bool,
) -> Result<(), String> {
    let de = if active { "" } else { "de" };
    let time = Utc::now().naive_utc();
    let npcs = diesel::update(npc_classes::table)
        .filter(npc_classes::dsl::name.eq(name.clone()))
        .set((
            npc_classes::dsl::active.eq(if active { 1 } else { 0 }),
            npc_classes::dsl::next_tick.eq(time),
        ))
        .execute(connection)
        .map_err(|e| format!("could not {}activate npc: {}", de, e.to_string()))?;
    match npcs {
        0 => Err(format!("could not find npc: {}", name)),
        1 => Ok(()),
        _ => Err(format!("schema violation? {} npcs {}activated", npcs, de)),
    }
}

pub fn change_name(
    connection: &SqliteConnection,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    let npcs = diesel::update(npc_classes::table)
        .filter(npc_classes::dsl::name.eq(old_name.clone()))
        .set(npc_classes::dsl::name.eq(new_name))
        .execute(connection)
        .map_err(|e| format!("could not change npcs name: {}", e.to_string()))?;
    match npcs {
        0 => Err(format!("could not find npc: {}", old_name)),
        1 => Ok(()),
        _ => Err(format!("schema violation? {} npcs renamed", npcs)),
    }
}

pub fn change_freq(connection: &SqliteConnection, name: String, freq: i32) -> Result<(), String> {
    let npcs = diesel::update(npc_classes::table)
        .filter(npc_classes::dsl::name.eq(name.clone()))
        .set(npc_classes::dsl::commonality.eq(freq))
        .execute(connection)
        .map_err(|e| format!("could not change npcs frequency: {}", e.to_string()))?;
    match npcs {
        0 => Err(format!("could not find npc: {}", name)),
        1 => Ok(()),
        _ => Err(format!("schema violation? {} npcs modified", npcs)),
    }
}

pub fn remove_class(connection: &SqliteConnection, name: String) -> Result<(), String> {
    let npcs = diesel::delete(npc_classes::table)
        .filter(npc_classes::dsl::name.eq(name.clone()))
        .execute(connection)
        .map_err(|e| format!("could not delete npc: {}", e.to_string()))?;
    match npcs {
        0 => Err(format!("could not find npc: {}", name)),
        1 => Ok(()),
        _ => Err(format!("schema violation? {} npcs deleted", npcs)),
    }
}

pub fn remove_instance(connection: &SqliteConnection, id: i32) -> Result<(), String> {
    let npcs = diesel::delete(npc_instances::table)
        .filter(npc_instances::dsl::id.eq(id))
        .execute(connection)
        .map_err(|e| format!("could not delete instance: {}", e.to_string()))?;
    match npcs {
        0 => Err(format!("could not find instance: {}", id)),
        1 => Ok(()),
        _ => Err(format!("schema violation? {} instances deleted", npcs)),
    }
}

pub fn get_classes(connection: &SqliteConnection) -> Result<Vec<NpcClass>, String> {
    npc_classes::table.load(connection).map_err(|e| {
        format!(
            "could not query database for npc classes: {}",
            e.to_string()
        )
    })
}

pub fn get_instances(
    connection: &SqliteConnection,
) -> Result<Vec<(NpcInstance, NpcClass)>, String> {
    npc_instances::table
        .inner_join(npc_classes::table)
        .load(connection)
        .map_err(|e| {
            format!(
                "could not query database for npc instances: {}",
                e.to_string()
            )
        })
}

pub fn get_aliases(connection: &SqliteConnection) -> Result<Vec<Alias>, String> {
    aliases::table
        .load(connection)
        .map_err(|e| format!("could not query database for aliases: {}", e.to_string()))
}

pub fn get_constant(connection: &SqliteConnection, key: String) -> Result<Constant, String> {
    let mut result: Vec<Constant> = constants::table
        .filter(constants::dsl::key.eq(key.clone()))
        .load(connection)
        .map_err(|e| {
            format!(
                "could not query database for constant {}: {}",
                key,
                e.to_string()
            )
        })?;
    match result.len() {
        0 => Err(format!("could not find constant: {}", key)),
        1 => Ok(result.pop().unwrap()),
        _ => Err(format!(
            "schema violation: multiple values found for key: {}",
            key
        )),
    }
}

pub fn add_alias(
    connection: &SqliteConnection,
    command: String,
    alias: String,
) -> Result<(), String> {
    let alias = NewAlias {
        command: &command,
        alias: &alias,
    };
    diesel::insert_into(aliases::table)
        .values(&alias)
        .execute(connection)
        .map_err(|e| format!("could not insert new npc: {}", e.to_string()))?;
    Ok(())
}

pub fn remove_alias(connection: &SqliteConnection, alias: String) -> Result<(), String> {
    let result = diesel::delete(aliases::table)
        .filter(aliases::dsl::alias.eq(alias.clone()))
        .execute(connection)
        .map_err(|e| format!("could not delete alias: {}", e.to_string()))?;
    match result {
        0 => Err(format!("could not find alias: {}", alias)),
        1 => Ok(()),
        _ => Err(format!("schema violation? {} aliases deleted", result)),
    }
}

pub fn change_constant(
    connection: &SqliteConnection,
    key: String,
    value: Option<String>,
) -> Result<(), String> {
    let result = diesel::update(constants::table)
        .filter(constants::dsl::key.eq(key.clone()))
        .set(constants::dsl::value.eq(value))
        .execute(connection)
        .map_err(|e| {
            format!(
                "could not change value of {} constant: {}",
                key,
                e.to_string()
            )
        })?;
    match result {
        0 => Err(format!("constant not found: {}", key)),
        1 => Ok(()),
        _ => Err(format!("schema violation? multiple \"{}\" constants", key)),
    }
}
