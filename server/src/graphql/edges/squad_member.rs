use crate::graphql::PageInfo;
use crate::graphql::nodes::Person;

#[async_graphql::SimpleObject]
pub struct SquadMemberEdge {
    pub cursor: String,
    pub node: Person,
}

#[async_graphql::SimpleObject]
pub struct SquadMemberConnection {
    pub edges: Vec<SquadMemberEdge>,
    pub page_info: PageInfo,
}