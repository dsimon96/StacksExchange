table! {
    balance (id) {
        id -> Int4,
        node_id -> Int4,
        person_id -> Int4,
        squad_id -> Int4,
    }
}

table! {
    node (id) {
        id -> Int4,
        uid -> Uuid,
        node_type -> crate::db::models::NodeTypeMapping,
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
    squad (id) {
        id -> Int4,
        node_id -> Int4,
        display_name -> Varchar,
    }
}

table! {
    txn (id) {
        id -> Int4,
        node_id -> Int4,
        squad_id -> Int4,
    }
}

table! {
    txn_part (id) {
        id -> Int4,
        txn_id -> Int4,
        balance_id -> Int4,
        balance_change_cents -> Int4,
    }
}

joinable!(balance -> node (node_id));
joinable!(balance -> person (person_id));
joinable!(balance -> squad (squad_id));
joinable!(person -> node (node_id));
joinable!(squad -> node (node_id));
joinable!(txn -> node (node_id));
joinable!(txn -> squad (squad_id));
joinable!(txn_part -> balance (balance_id));
joinable!(txn_part -> txn (txn_id));

allow_tables_to_appear_in_same_query!(balance, node, person, squad, txn, txn_part,);
