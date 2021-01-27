use super::super::{nodes::Transaction, PageInfo};
use crate::db::{
    models,
    schema::{node, txn},
    Pool,
};
use async_graphql::Result;
use diesel::prelude::*;
use tokio_diesel::*;

#[derive(async_graphql::SimpleObject)]
pub struct SquadTransactionEdge {
    pub cursor: String,
    pub node: Transaction,
}

#[derive(async_graphql::SimpleObject)]
pub struct SquadTransactionConnection {
    pub edges: Vec<SquadTransactionEdge>,
    pub page_info: PageInfo,
}

impl SquadTransactionConnection {
    pub async fn by_squad_id(pool: &Pool, squad_id: i32) -> Result<SquadTransactionConnection> {
        Ok(node::table
            .inner_join(txn::table)
            .filter(txn::squad_id.eq(squad_id))
            .get_results_async::<models::Transaction>(pool)
            .await
            .map(|results| SquadTransactionConnection {
                edges: results
                    .into_iter()
                    .map(|transaction| SquadTransactionEdge {
                        cursor: String::from(""),
                        node: transaction.into(),
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
