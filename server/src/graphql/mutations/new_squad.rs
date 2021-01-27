use super::super::nodes::Squad;
use crate::db::{
    models,
    schema::{node, squad},
    Pool,
};
use async_graphql::Result;
use diesel::prelude::*;
use tokio_diesel::*;
use uuid::Uuid;

#[derive(async_graphql::InputObject)]
pub struct NewSquadInput {
    pub display_name: String,
}

#[derive(async_graphql::SimpleObject)]
pub struct NewSquadPayload {
    pub squad: Squad,
}

pub async fn new_squad(pool: &Pool, input: NewSquadInput) -> Result<NewSquadPayload> {
    Ok(pool
        .transaction(move |conn| {
            let new_node = models::NewNode {
                uid: Uuid::new_v4(),
                node_type: models::NodeType::Squad,
            };

            let node = diesel::insert_into(node::table)
                .values(new_node)
                .get_result::<models::Node>(conn)?;

            let new_squad = models::NewSquad {
                node_id: node.id,
                display_name: &input.display_name,
            };

            diesel::insert_into(squad::table)
                .values(&new_squad)
                .get_result::<models::SquadDetail>(conn)
                .map(|detail| NewSquadPayload {
                    squad: models::Squad { node, detail }.into(),
                })
        })
        .await?)
}
