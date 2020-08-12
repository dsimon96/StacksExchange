table! {
    node (id) {
        id -> Int4,
        uid -> Uuid,
    }
}

table! {
    person (id) {
        id -> Int4,
        node_id -> Int4,
        display_name -> Varchar,
        email -> Varchar,
        first_name -> Varchar,
        last_name -> Varchar,
    }
}

table! {
    person_squad_connection (id) {
        id -> Int4,
        person_id -> Int4,
        squad_id -> Int4,
    }
}

table! {
    squad (id) {
        id -> Int4,
        node_id -> Int4,
        display_name -> Varchar,
    }
}

joinable!(person -> node (node_id));
joinable!(person_squad_connection -> person (person_id));
joinable!(person_squad_connection -> squad (squad_id));
joinable!(squad -> node (node_id));

allow_tables_to_appear_in_same_query!(
    node,
    person,
    person_squad_connection,
    squad,
);
