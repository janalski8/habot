use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    ShowInstances,
    ShowInstancesVerbose,
    ShowClasses,
    AddClass(String, i32, bool),
    RemoveInstances,
    RemoveInstance(i32),
    RemoveClass(String),
    ChangeClassName(String, String),
    ChangeClassFreq(String, i32),
    ChangeClassActive(String, bool),
    FastForward(i32),
}

pub fn parse_command(mut command: Vec<String>) -> Result<Command, String> {
    command.reverse();

    let command_group: String = command.pop().ok_or_else(|| {
        "available commands: show | add | remove | change | fast-forward [minutes: integer]"
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
    let target = command
        .pop()
        .ok_or_else(|| "available commands: instance verbose? | class".to_string())?;

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
        "available commands: class [name] [freq: integer] [active?: true|false]".to_string()
    })?;

    match target.as_ref() {
        "class" => {
            let name = command.pop();
            let frequency = command.pop().map(|s| i32::from_str(&s));
            let active = bool::from_str(&command.pop().unwrap_or_else(|| "true".to_string()));
            match (name, frequency, active) {
                (None, _, _) => Err(format!("name unspecified")),
                (_, None, _) => Err(format!("frequency unspecified")),
                (_, Some(Err(_)), _) => Err(format!("invalid frequency (integer)")),
                (_, _, Err(_)) => Err(format!("invalid active/inactive state (true/false)")),
                (Some(name), Some(Ok(frequency)), Ok(active)) => {
                    Ok(Command::AddClass(name, frequency, active))
                }
            }
        }
        arg => Err(format!("invalid command arguments: {}", arg)),
    }
}

pub fn parse_remove(mut command: Vec<String>) -> Result<Command, String> {
    let target = command.pop().ok_or_else(|| {
        "available commands: class [name] | instance [id: integer] | all_instances".to_string()
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
        arg => Err(format!("invalid command arguments: {}", arg)),
    }
}

pub fn parse_change(mut command: Vec<String>) -> Result<Command, String> {
    let target = command.pop().ok_or_else(|| {
        "available commands: \
         class [old_name] name [name] | \
         class [name] freq [freq: i32] | \
         class [name] active [active: true|false]"
            .to_string()
    })?;

    match target.as_ref() {
        "class" => {
            let name = command
                .pop()
                .ok_or_else(|| "class name missing".to_string())?;
            let key = command
                .pop()
                .ok_or_else(|| "available keys: name | frequency | active".to_string())?;
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
                arg => Err(format!("invalid key: {}", arg)),
            }
        }
        arg => Err(format!("invalid command arguments: {}", arg)),
    }
}