use super::super::{nodes::Transaction, PageInfo};
use crate::db::{
    models,
    schema::{node, txn, txn_part},
    Pool,
};
use async_graphql::Result;
use diesel::prelude::*;
use tokio_diesel::*;

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
    ) -> Result<BalanceTransactionConnection> {
        Ok(txn_part::table
            .filter(txn_part::balance_id.eq(balance_id))
            .inner_join(
                node::table
                    .inner_join(txn::table)
                    .on(txn_part::txn_id.eq(txn::id)),
            )
            .get_results_async::<(models::TransactionPart, models::Transaction)>(pool)
            .await
            .map(|results| BalanceTransactionConnection {
                edges: results
                    .into_iter()
                    .map(|(transaction_part, transaction)| BalanceTransactionEdge {
                        cursor: String::from(""),
                        node: transaction.into(),
                        balance_change_cents: transaction_part.balance_change_cents,
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
