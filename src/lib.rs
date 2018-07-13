#![feature(tool_attributes)]
#[cfg_attr(rustfmt, rustfmt_skip)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;

extern crate byteorder;
extern crate chrono;
extern crate core;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate serenity;
extern crate shlex;

use diesel::sqlite::SqliteConnection;
use diesel::Connection;

pub mod command;
pub mod execute;
pub mod models;
pub mod queries;
pub mod schema;
pub mod timing;

pub fn establish_connection(url: &str) -> Result<SqliteConnection, String> {
    SqliteConnection::establish(&url).map_err(|_e| format!("Error connecting to {}", url))
}
