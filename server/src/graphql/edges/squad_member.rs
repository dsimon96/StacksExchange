use super::super::{nodes::Person, PageInfo};

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
