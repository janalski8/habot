table! {
    npc_classes (id) {
        id -> Integer,
        name -> Text,
        commonality -> Integer,
        next_tick -> Timestamp,
        active -> Integer,
    }
}

table! {
    npc_instances (id) {
        id -> Integer,
        class -> Integer,
        active_until -> Timestamp,
    }
}

joinable!(npc_instances -> npc_classes (class));

allow_tables_to_appear_in_same_query!(npc_classes, npc_instances,);
