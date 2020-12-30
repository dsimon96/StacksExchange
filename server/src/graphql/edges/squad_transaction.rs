use super::{
    super::nodes::Transaction,
    util::{Cursor, CursorDetail},
};
use crate::db::{
    models,
    schema::{node, txn},
    Pool,
};
use async_graphql::{
    connection::{query_with, CursorType, PageInfo},
    Result,
};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tokio_diesel::*;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub enum SquadTransactionCursorDetail {
    ByAscendingId(Uuid),
}

impl CursorDetail for SquadTransactionCursorDetail {}

pub type SquadTransactionCursor = Cursor<SquadTransactionCursorDetail>;

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
    pub async fn by_squad_id(
        pool: &Pool,
        squad_id: i32,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<SquadTransactionConnection> {
        query_with(
            after,
            before,
            first,
            last,
            |_after: Option<SquadTransactionCursor>,
             _before: Option<SquadTransactionCursor>,
             _first,
             _last| async move {
                let transactions = node::table
                    .inner_join(txn::table)
                    .filter(txn::squad_id.eq(squad_id))
                    .order(txn::id.asc())
                    .get_results_async::<models::Transaction>(pool)
                    .await?;

                let edges: Vec<SquadTransactionEdge> = transactions
                    .into_iter()
                    .map(|transaction| SquadTransactionEdge {
                        cursor: Cursor(SquadTransactionCursorDetail::ByAscendingId(
                            transaction.node.uid,
                        ))
                        .encode_cursor(),
                        node: transaction.into(),
                    })
                    .collect();

                let start_cursor = edges.first().map(|edge| edge.cursor.clone());
                let end_cursor = edges.last().map(|edge| edge.cursor.clone());

                Ok(SquadTransactionConnection {
                    edges,
                    page_info: PageInfo {
                        has_previous_page: false,
                        has_next_page: false,
                        start_cursor,
                        end_cursor,
                    },
                })
            },
        )
        .await
    }
}
