use crate::graphql::nodes::{Person, Squad};
use async_graphql::ID;

#[async_graphql::InputObject]
pub struct AddPersonToSquadInput {
    pub person_id: ID,
    pub squad_id: ID,
}

#[async_graphql::SimpleObject]
pub struct AddPersonToSquadPayload {
    pub person: Person,
    pub squad: Squad,
}