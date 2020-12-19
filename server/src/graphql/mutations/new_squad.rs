use crate::graphql::nodes::Squad;

#[async_graphql::InputObject]
pub struct NewSquadInput {
    pub display_name: String,
}

#[async_graphql::SimpleObject]
pub struct NewSquadPayload {
    pub squad: Squad,
}