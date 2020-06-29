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

joinable!(person -> node (node_id));

allow_tables_to_appear_in_same_query!(
    node,
    person,
);
