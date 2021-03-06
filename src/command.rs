use std::collections::HashMap;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    RemoveAliasCommand(String),
    AddAliasCommand(String, Vec<String>),
    ShowInstances,
    ShowInstancesVerbose,
    ShowClasses,
    AddClass(String, i32, bool, bool),
    RemoveInstances,
    RemoveInstance(i32),
    RemoveClass(String),
    ChangeClassName(String, String),
    ChangeClassFreq(String, i32),
    ChangeClassActive(String, bool),
    ChangeClassUnique(String, bool),
    FastForward(i32),
    ChangeStarter(String),
    AddAdmin(u64),
    RemoveAdmin(u64),
}

impl Command {
    pub fn is_public(&self) -> bool {
        match self {
            &Command::ShowInstances |
            &Command::ShowInstancesVerbose => true,
            _ => false
        }
    }
}

pub fn parse_aliased(
    mut command: Vec<String>,
    aliases: HashMap<String, Vec<String>>,
) -> Result<Command, String> {
    if let Some(pattern) = command.get(0).and_then(|c| aliases.get(c)) {
        command.reverse();
        command.pop();
        let mut output = pattern
            .into_iter()
            .map(|part| match part.as_ref() {
                "?" => command
                    .pop()
                    .ok_or_else(|| "alias requires more arguments!".to_string()),
                fixed => Ok(fixed.to_owned()),
            })
            .collect::<Result<Vec<_>, _>>();
        let mut output = output?;
        if command.len() > 0 {
            return Err("alias didn't use all the arguments!".to_string());
        }
        parse_command(output)
    } else {
        parse_command(command)
    }
}

pub fn parse_command(mut command: Vec<String>) -> Result<Command, String> {
    command.reverse();

    let command_group: String = command.pop().ok_or_else(|| {
        "available commands: \
         show | \
         add | \
         remove | \
         change | \
         fast-forward [minutes: integer]"
            .to_string()
    })?;

    match command_group.as_ref() {
        "show" => parse_show(command),
        "add" => parse_add(command),
        "remove" => parse_remove(command),
        "change" => parse_change(command),
        "fast-forward" => match command.pop().map(|s| i32::from_str(&s)) {
            None => Err("missing minutes count".to_string()),
            Some(Err(e)) => Err(format!("invalid minutes count format: {}", e.to_string())),
            Some(Ok(minutes)) => Ok(Command::FastForward(minutes)),
        },
        arg => Err(format!("invalid command: {}", arg)),
    }
}

pub fn parse_show(mut command: Vec<String>) -> Result<Command, String> {
    let target = command.pop().ok_or_else(|| {
        "available commands: \n\
         instance verbose? | \
         class"
            .to_string()
    })?;

    match target.as_ref() {
        "instance" => {
            let modifier = command.pop();
            match modifier.as_ref().map(String::as_ref) {
                None => Ok(Command::ShowInstances),
                Some("verbose") => Ok(Command::ShowInstancesVerbose),
                Some(arg) => Err(format!("invalid command: {}", arg)),
            }
        }
        "class" => Ok(Command::ShowClasses),
        arg => Err(format!("invalid command arguments: {}", arg)),
    }
}

pub fn parse_add(mut command: Vec<String>) -> Result<Command, String> {
    let target = command.pop().ok_or_else(|| {
        "available commands:\n\
         class [name] [freq: integer] [active?: true|false] [unique?: true|false]\n\
         alias [alias] [command]\n\
         admin [admin: mention]"
            .to_string()
    })?;

    match target.as_ref() {
        "class" => {
            let name = command.pop();
            let frequency = command.pop().map(|s| i32::from_str(&s));
            let active = bool::from_str(&command.pop().unwrap_or_else(|| "true".to_string()));
            let unique = bool::from_str(&command.pop().unwrap_or_else(|| "false".to_string()));
            match (name, frequency, active, unique) {
                (None, _, _, _) => Err(format!("name unspecified")),
                (_, None, _, _) => Err(format!("frequency unspecified")),
                (_, Some(Err(_)), _, _) => Err(format!("invalid frequency (integer)")),
                (_, _, Err(_), _) => Err(format!("invalid active/inactive state (true/false)")),
                (_, _, _, Err(_)) => Err(format!("invalid unique state (true/false)")),
                (Some(name), Some(Ok(frequency)), Ok(active), Ok(unique)) => {
                    Ok(Command::AddClass(name, frequency, active, unique))
                }
            }
        }
        "alias" => {
            let alias = command
                .pop()
                .ok_or_else(|| format!("missing alias string"))?;
            if command.len() == 0 {
                return Err(format!("missing command string"));
            }
            command.reverse();
            Ok(Command::AddAliasCommand(alias, command))
        }
        "admin" => {
            let raw_id = command
                .pop()
                .ok_or_else(|| format!("missing admin (requires mention)"))?;
            let slice = raw_id
                .get(2..(raw_id.len() - 1))
                .ok_or_else(|| format!("could not parse admin mention: {}", raw_id))?;
            let id = u64::from_str(slice)
                .map_err(|e| format!("could not parse admin mention: {}", e.to_string()))?;
            Ok(Command::AddAdmin(id))
        }
        arg => Err(format!("invalid command arguments: {}", arg)),
    }
}

pub fn parse_remove(mut command: Vec<String>) -> Result<Command, String> {
    let target = command.pop().ok_or_else(|| {
        "available commands:\n\
         class [name]\n\
         instance [id: integer]\n\
         all_instances\n\
         alias [alias]\n\
         admin [admin: mention]"
            .to_string()
    })?;

    match target.as_ref() {
        "class" => {
            let name = command.pop();
            match name {
                Some(name) => Ok(Command::RemoveClass(name)),
                None => Err(format!("name unspecified")),
            }
        }
        "instance" => {
            let id_str = command.pop().ok_or_else(|| "id unspecified".to_string())?;
            let id =
                i32::from_str(&id_str).map_err(|e| format!("invalid id (integer) format: {}", e))?;
            Ok(Command::RemoveInstance(id))
        }
        "all_instances" => Ok(Command::RemoveInstances),
        "alias" => {
            let alias = command
                .pop()
                .ok_or_else(|| format!("alias string missing"))?;
            Ok(Command::RemoveAliasCommand(alias))
        }
        "admin" => {
            let raw_id = command
                .pop()
                .ok_or_else(|| format!("missing admin (requires mention)"))?;
            let slice = raw_id
                .get(2..(raw_id.len() - 1))
                .ok_or_else(|| format!("could not parse admin mention: {}", raw_id))?;
            let id = u64::from_str(slice)
                .map_err(|e| format!("could not parse admin mention: {}", e.to_string()))?;
            Ok(Command::RemoveAdmin(id))
        }
        arg => Err(format!("invalid command arguments: {}", arg)),
    }
}

pub fn parse_change(mut command: Vec<String>) -> Result<Command, String> {
    let target = command.pop().ok_or_else(|| {
        "available commands:\n\
         class [old_name] name [name]\n\
         class [name] freq [freq: i32]\n\
         class [name] active [active: true|false]\n\
         class [name] unique [unique: true|false]\n\
         starter [starter_string]"
            .to_string()
    })?;

    match target.as_ref() {
        "class" => {
            let name = command
                .pop()
                .ok_or_else(|| "class name missing".to_string())?;
            let key = command
                .pop()
                .ok_or_else(|| "available keys: name | freq | active | unique".to_string())?;
            let value = command
                .pop()
                .ok_or_else(|| "new value missing".to_string())?;
            match key.as_ref() {
                "name" => Ok(Command::ChangeClassName(name, value)),
                "freq" => {
                    let freq = i32::from_str(&value).map_err(|e| {
                        format!("invalid frequency (integer) format: {}", e.to_string())
                    })?;
                    Ok(Command::ChangeClassFreq(name, freq))
                }
                "active" => {
                    let active = bool::from_str(&value).map_err(|e| {
                        format!("invalid active (boolean) format: {}", e.to_string())
                    })?;
                    Ok(Command::ChangeClassActive(name, active))
                }
                "unique" => {
                    let unique = bool::from_str(&value).map_err(|e| {
                        format!("invalid unique (boolean) format: {}", e.to_string())
                    })?;
                    Ok(Command::ChangeClassUnique(name, unique))
                }
                arg => Err(format!("invalid key: {}", arg)),
            }
        }
        "starter" => {
            let starter = command
                .pop()
                .ok_or_else(|| format!("new starter string missing"))?;
            Ok(Command::ChangeStarter(starter))
        }
        arg => Err(format!("invalid command arguments: {}", arg)),
    }
}
