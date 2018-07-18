extern crate diesel;
extern crate habot;
extern crate serenity;
extern crate shlex;

use diesel::sqlite::SqliteConnection;
use habot::command::parse_command;
use habot::establish_connection;
use habot::execute::execute_command;
use habot::queries::get_constant;
use serenity::client::Context;
use serenity::client::EventHandler;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::Client;
use std::env;
use habot::queries::get_aliases;
use habot::command::parse_aliased;

struct Handler {
    url: String,
    starter: String,
}

impl Handler {
    fn process(&self, connection: &SqliteConnection, text: String) -> Result<String, String> {
        let args = shlex::split(&text).ok_or_else(|| "malformed arguments string".to_string())?;
        let aliases = get_aliases(connection)?;
        let cmd = parse_aliased(args, aliases)?;
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
            Ok(s) => s.map(|c| c.value).unwrap_or_else(|| self.starter.clone()),
            Err(e) => {
                println!("{}", e);
                return;
            }
        };

        if let Some(text) = try_strip_of(&starter, &msg.content) {
            let result = match self.process(&connection, text) {
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

fn try_strip_of(starter: &str, text: &str) -> Option<String> {
    let mut parts = text.splitn(2, &starter);
    match parts.next().map(str::is_empty) {
        None | Some(false) => None,
        Some(true) => parts.next().map(str::to_owned)
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
