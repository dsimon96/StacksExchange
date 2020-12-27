use super::schema::{balance, node, person, squad};
use diesel_derive_enum::DbEnum;
use uuid::Uuid;

#[derive(Debug, DbEnum)]
pub enum NodeType {
    Person,
    Squad,
    Balance,
}

#[derive(Queryable, Identifiable)]
#[table_name = "node"]
pub struct Node {
    pub id: i32,
    pub uid: Uuid,
    pub node_type: NodeType,
}

#[derive(Insertable)]
#[table_name = "node"]
pub struct NewNode {
    pub uid: Uuid,
    pub node_type: NodeType,
}

#[derive(Queryable, Identifiable)]
#[table_name = "person"]
pub struct PersonDetail {
    pub id: i32,
    pub node_id: i32,
    pub display_name: String,
    pub email: String,
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
    pub id: i32,
    pub node_id: i32,
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

#[derive(Insertable)]
#[table_name = "squad"]
pub struct NewSquad<'a> {
    pub node_id: i32,
    pub display_name: &'a str,
}

#[derive(Queryable, Identifiable)]
#[table_name = "balance"]
pub struct BalanceDetail {
    pub id: i32,
    pub node_id: i32,
    pub person_id: i32,
    pub squad_id: i32,
}

#[derive(Queryable)]
pub struct Balance {
    pub node: Node,
    pub detail: BalanceDetail,
}

#[derive(Insertable)]
#[table_name = "balance"]
pub struct NewBalance {
    pub node_id: i32,
    pub person_id: i32,
    pub squad_id: i32,
}
