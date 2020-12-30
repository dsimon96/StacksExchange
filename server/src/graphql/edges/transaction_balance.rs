use super::{
    super::nodes::Balance,
    util::{Cursor, CursorDetail},
};
use crate::db::{
    models,
    schema::{balance, node, txn_part},
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
pub enum TransactionBalanceCursorDetail {
    ByAscendingId(Uuid),
}

impl CursorDetail for TransactionBalanceCursorDetail {}

pub type TransactionBalanceCursor = Cursor<TransactionBalanceCursorDetail>;

#[derive(async_graphql::SimpleObject)]
pub struct TransactionBalanceEdge {
    pub cursor: String,
    pub node: Balance,
    pub balance_change_cents: i32,
}

#[derive(async_graphql::SimpleObject)]
pub struct TransactionBalanceConnection {
    pub edges: Vec<TransactionBalanceEdge>,
    pub page_info: PageInfo,
}

impl TransactionBalanceConnection {
    pub async fn by_transaction_id(
        pool: &Pool,
        transaction_id: i32,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<TransactionBalanceConnection> {
        query_with(
            after,
            before,
            first,
            last,
            |_after: Option<TransactionBalanceCursor>,
             _before: Option<TransactionBalanceCursor>,
             _first,
             _last| async move {
                let balances = txn_part::table
                    .filter(txn_part::txn_id.eq(transaction_id))
                    .inner_join(
                        node::table
                            .inner_join(balance::table)
                            .on(txn_part::balance_id.eq(balance::id)),
                    )
                    .order(balance::id.asc())
                    .get_results_async::<(models::TransactionPart, models::Balance)>(pool)
                    .await?;

                let edges: Vec<TransactionBalanceEdge> = balances
                    .into_iter()
                    .map(|(transaction_part, balance)| TransactionBalanceEdge {
                        cursor: Cursor(TransactionBalanceCursorDetail::ByAscendingId(
                            balance.node.uid,
                        ))
                        .encode_cursor(),
                        node: balance.into(),
                        balance_change_cents: transaction_part.balance_change_cents,
                    })
                    .collect();

                let start_cursor = edges.first().map(|edge| edge.cursor.clone());
                let end_cursor = edges.last().map(|edge| edge.cursor.clone());

                Ok(TransactionBalanceConnection {
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
