extern crate habot;
extern crate serenity;
extern crate shlex;
extern crate diesel;

use habot::command::parse_command;
use habot::establish_connection;
use habot::execute::execute_command;
use serenity::client::Context;
use serenity::client::EventHandler;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::Client;
use std::env;
use habot::queries::get_constant;
use diesel::sqlite::SqliteConnection;

struct Handler {
    url: String,
    starter: String,
}

impl Handler {
    fn process(&self, connection: &SqliteConnection, text: String) -> Result<String, String> {
        let mut parts = text.splitn(2, &self.starter);
        parts
            .next()
            .ok_or_else(|| "unable to parse command".to_string())?;
        let raw_args = parts
            .next()
            .ok_or_else(|| "unable to parse command".to_string())?;
        let args = shlex::split(raw_args).ok_or_else(|| "malformed arguments string".to_string())?;
        let cmd = parse_command(args)?;
        let result = execute_command(connection, cmd);
        result
    }
}

impl EventHandler for Handler {
    fn message(&self, _: Context, msg: Message) {

        let connection = match establish_connection(&self.url) {
            Ok(c) => c,
            Err(e) => {
                println!("could not connect to database: {}", e);
                return;
            }
        };

        let starter = match get_constant(&connection, "starter".to_string()) {
            Ok(s) => s.value.unwrap_or_else(|| "!".to_string()),
            Err(e) => {
                println!("{}", e);
                return;
            },
        };

        if msg.content.starts_with(&starter) {
            let result = match self.process(&connection, msg.content.clone()) {
                Err(e) => msg.channel_id.say(format!("Error: {}", e)),
                Ok(r) => msg.channel_id.say(r),
            };
            match result {
                Ok(_) => {}
                Err(e) => println!("could not send response: {}", e),
            }
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() -> Result<(), String> {
    let mut args = env::args().collect::<Vec<_>>();
    args.remove(0);
    let url = args.remove(0);
    let handler = Handler {
        url,
        starter: "!".to_string(),
    };

    let token = "NDY2MTY3NzM4OTEwNjM4MDgw.DiYIKQ.ScJVGtK4HL4CSdTsrrrhFWKjxmg";
    let mut client = Client::new(token, handler).expect("Err creating client");
    client
        .start()
        .map_err(|e| format!("could not start bot: {}", e.to_string()))?;
    Ok(())
}
