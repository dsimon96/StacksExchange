use super::{
    super::nodes::Transaction,
    util::{Cursor, CursorDetail},
};
use crate::db::{
    models,
    schema::{node, txn, txn_part},
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
pub enum BalanceTransactionCursorDetail {
    ByAscendingId(Uuid),
}

impl CursorDetail for BalanceTransactionCursorDetail {}

pub type BalanceTransactionCursor = Cursor<BalanceTransactionCursorDetail>;

#[derive(async_graphql::SimpleObject)]
pub struct BalanceTransactionEdge {
    pub cursor: String,
    pub node: Transaction,
    pub balance_change_cents: i32,
}

#[derive(async_graphql::SimpleObject)]
pub struct BalanceTransactionConnection {
    pub edges: Vec<BalanceTransactionEdge>,
    pub page_info: PageInfo,
}

impl BalanceTransactionConnection {
    pub async fn by_balance_id(
        pool: &Pool,
        balance_id: i32,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<BalanceTransactionConnection> {
        query_with(
            after,
            before,
            first,
            last,
            |_after: Option<BalanceTransactionCursor>,
             _before: Option<BalanceTransactionCursor>,
             _first,
             _last| async move {
                let transactions = txn_part::table
                    .filter(txn_part::balance_id.eq(balance_id))
                    .inner_join(
                        node::table
                            .inner_join(txn::table)
                            .on(txn_part::txn_id.eq(txn::id)),
                    )
                    .order(txn::id.asc())
                    .get_results_async::<(models::TransactionPart, models::Transaction)>(pool)
                    .await?;

                let edges: Vec<BalanceTransactionEdge> = transactions
                    .into_iter()
                    .map(|(transaction_part, transaction)| BalanceTransactionEdge {
                        cursor: Cursor(BalanceTransactionCursorDetail::ByAscendingId(
                            transaction.node.uid,
                        ))
                        .encode_cursor(),
                        node: transaction.into(),
                        balance_change_cents: transaction_part.balance_change_cents,
                    })
                    .collect();

                let start_cursor = edges.first().map(|edge| edge.cursor.clone());
                let end_cursor = edges.last().map(|edge| edge.cursor.clone());

                Ok(BalanceTransactionConnection {
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
