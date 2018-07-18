extern crate habot;

use habot::command::parse_command;
use habot::establish_connection;
use habot::execute::execute_command;
use std::env;

fn main() -> Result<(), String> {
    let mut args = env::args().collect::<Vec<_>>();
    args.remove(0);

    let url = args.remove(0);
    let connection = establish_connection(&url)?;

    let command = parse_command(args);

    let result = match command {
        Err(e) => {
            return Err(e);
        }
        Ok(command) => execute_command(&connection, command),
    };

    match result {
        Err(e) => {
            return Err(e);
        }
        Ok(result) => println!("{}", result),
    }

    Ok(())
}
