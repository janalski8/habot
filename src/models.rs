use chrono::NaiveDateTime;
use schema::{admins, aliases, constants, npc_classes, npc_instances};

#[derive(Serialize, Deserialize, Queryable, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NpcClass {
    pub id: i32,
    pub name: String,
    pub commonality: i32,
    pub next_tick: NaiveDateTime,
    pub active: i32,
    pub unique: i32
}

#[derive(Serialize, Deserialize, Queryable, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NpcInstance {
    pub id: i32,
    pub class: i32,
    pub active_until: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Queryable, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Alias {
    pub id: i32,
    pub alias: String,
    pub command: String,
}

#[derive(Serialize, Deserialize, Queryable, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Constant {
    pub id: i32,
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Queryable, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Admin {
    pub id: i32,
    pub user_id: String,
}

#[derive(Insertable, Debug)]
#[table_name = "npc_classes"]
pub struct NewNpcClass<'a> {
    pub name: &'a str,
    pub commonality: i32,
    pub next_tick: NaiveDateTime,
    pub active: i32,
    pub unique: i32
}

#[derive(Insertable, Debug)]
#[table_name = "npc_instances"]
pub struct NewNpcInstance {
    pub class: i32,
    pub active_until: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[table_name = "aliases"]
pub struct NewAlias<'a> {
    pub alias: &'a str,
    pub command: &'a str,
}

#[derive(Insertable, Debug)]
#[table_name = "constants"]
pub struct NewConstant<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

#[derive(Insertable, Debug)]
#[table_name = "admins"]
pub struct NewAdmin<'a> {
    pub user_id: &'a str,
}
