use super::schema::{node, person, person_squad_connection, squad};
use uuid::Uuid;

#[derive(Queryable, Identifiable)]
#[table_name = "node"]
pub struct Node {
    pub id: i32,
    pub uid: Uuid,
}

#[derive(Queryable, Identifiable)]
#[table_name = "person"]
pub struct PersonDetail {
    pub node_id: i32,
    pub id: i32,
    pub email: String,
    pub display_name: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Queryable)]
pub struct Person {
    pub node: Node,
    pub detail: PersonDetail,
}

#[derive(Queryable, Identifiable)]
#[table_name = "squad"]
pub struct SquadDetail {
    pub node_id: i32,
    pub id: i32,
    pub display_name: String,
}

#[derive(Queryable)]
pub struct Squad {
    pub node: Node,
    pub detail: SquadDetail,
}

#[derive(Insertable)]
#[table_name = "person"]
pub struct NewPerson<'a> {
    pub node_id: i32,
    pub email: &'a str,
    pub display_name: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
}

#[derive(Identifiable, Queryable)]
#[table_name = "person_squad_connection"]
pub struct PersonSquadConnection {
    pub id: i32,
    pub person_id: i32,
    pub squad_id: i32,
}

#[derive(Insertable)]
#[table_name = "person_squad_connection"]
pub struct NewPersonSquadConnection {
    pub person_id: i32,
    pub squad_id: i32,
}
