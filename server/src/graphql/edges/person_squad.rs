use crate::graphql::PageInfo;
use crate::graphql::nodes::Squad;

#[async_graphql::SimpleObject]
pub struct PersonSquadEdge {
    pub cursor: String,
    pub node: Squad,
}

#[async_graphql::SimpleObject]
pub struct PersonSquadConnection {
    pub edges: Vec<PersonSquadEdge>,
    pub page_info: PageInfo,
}