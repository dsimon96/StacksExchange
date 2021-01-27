use super::super::{nodes::Balance, PageInfo};
use crate::db::{
    models,
    schema::{balance, node},
    Pool,
};
use async_graphql::Result;
use diesel::prelude::*;
use tokio_diesel::*;

#[derive(async_graphql::SimpleObject)]
pub struct SquadBalanceEdge {
    pub cursor: String,
    pub node: Balance,
}

#[derive(async_graphql::SimpleObject)]
pub struct SquadBalanceConnection {
    pub edges: Vec<SquadBalanceEdge>,
    pub page_info: PageInfo,
}

impl SquadBalanceConnection {
    pub async fn by_squad_id(pool: &Pool, squad_id: i32) -> Result<SquadBalanceConnection> {
        Ok(node::table
            .inner_join(balance::table)
            .filter(balance::squad_id.eq(squad_id))
            .get_results_async::<models::Balance>(pool)
            .await
            .map(|results| SquadBalanceConnection {
                edges: results
                    .into_iter()
                    .map(|balance| SquadBalanceEdge {
                        cursor: String::from(""),
                        node: balance.into(),
                    })
                    .collect(),
                page_info: PageInfo {
                    has_next_page: false,
                    has_previous_page: false,
                    start_cursor: String::from(""),
                    end_cursor: String::from(""),
                },
            })?)
    }
}
