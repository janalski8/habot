use chrono;
use chrono::Local;
use chrono::TimeZone;
use command::Command;
use diesel;
use diesel::sqlite::SqliteConnection;
use diesel::RunQueryDsl;
use queries::add_class;
use queries::change_active;
use queries::change_freq;
use queries::change_name;
use queries::get_classes;
use queries::get_instances;
use queries::remove_class;
use queries::remove_instance;
use schema::npc_instances;
use std::collections::HashMap;
use timing::fast_forward_instances;
use timing::update_instances;
use queries::add_alias;
use queries::remove_alias;
use queries::change_constant;

pub fn execute_command(connection: &SqliteConnection, command: Command) -> Result<String, String> {
    match command {
        Command::ShowInstances => {
            update_instances(connection)?;
            let mut counts: HashMap<String, usize> = HashMap::new();
            get_instances(connection)?
                .into_iter()
                .for_each(|(_instance, class)| {
                    let count = counts.entry(class.name).or_insert(0);
                    *count += 1;
                });
            let mut result = counts.into_iter().collect::<Vec<_>>();
            result.sort();
            let result = result
                .into_iter()
                .map(|(name, count)| format!("{}: {}", name, count))
                .collect::<Vec<String>>();
            Ok(if result.len() == 0 {
                "no npcs!".to_string()
            } else {
                result.join("\n")
            })
        }
        Command::ShowInstancesVerbose => {
            update_instances(connection)?;
            let mut result = get_instances(connection)?
                .into_iter()
                .map(|(instance, class)| {
                    format!(
                        "id: {}, {}, active until {}",
                        instance.id,
                        class.name,
                        Local
                            .from_utc_datetime(&instance.active_until)
                            .time()
                            .format("%H:%M:%S"),
                    )
                })
                .collect::<Vec<String>>();
            result.sort();
            Ok(if result.len() == 0 {
                "no npcs!".to_string()
            } else {
                result.join("\n")
            })
        }
        Command::ShowClasses => {
            let mut result = get_classes(connection)?;
            result.sort();
            let result = result
                .into_iter()
                .map(|i| {
                    format!(
                        "{}, frequency: {}, active: {}",
                        i.name,
                        i.commonality,
                        if i.active > 0 { true } else { false }
                    )
                })
                .collect::<Vec<String>>();
            Ok(if result.len() == 0 {
                "no npc classes!".to_string()
            } else {
                result.join("\n")
            })
        }
        Command::AddClass(name, freq, active) => {
            add_class(connection, name, freq, active).map(|()| "ok".to_string())
        }
        Command::AddCommandAlias(cmd, alias) => {
            add_alias(connection, cmd, alias).map(|()| "ok".to_string())
        }
        Command::RemoveInstances => diesel::delete(npc_instances::table)
            .execute(connection)
            .map(|c| format!("deleted instances: {}", c))
            .map_err(|e| format!("could not remove instances: {}", e)),
        Command::RemoveClass(name) => remove_class(connection, name).map(|()| "ok".to_string()),
        Command::RemoveInstance(id) => remove_instance(connection, id).map(|()| "ok".to_string()),
        Command::RemoveCommandAlias(alias) => remove_alias(connection, alias).map(|()| "ok".to_string()),
        Command::ChangeClassName(old, new) => {
            change_name(connection, old, new).map(|()| "ok".to_string())
        }
        Command::ChangeClassFreq(name, freq) => {
            change_freq(connection, name, freq).map(|()| "ok".to_string())
        }
        Command::ChangeClassActive(name, active) => {
            change_active(connection, name, active).map(|()| "ok".to_string())
        }
        Command::ChangeStarter(starter) => {
            change_constant(connection, "starter".to_string(), Some(starter)).map(|()| "ok".to_string())
        }
        Command::FastForward(minutes) => {
            fast_forward_instances(connection, chrono::Duration::minutes(minutes as i64))
                .map(|()| "ok".to_string())
        }
    }
}
