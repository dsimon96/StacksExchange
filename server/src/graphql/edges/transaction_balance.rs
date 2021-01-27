use super::super::{nodes::Balance, PageInfo};
use crate::db::{
    models,
    schema::{balance, node, txn_part},
    Pool,
};
use async_graphql::Result;
use diesel::prelude::*;
use tokio_diesel::*;

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
    ) -> Result<TransactionBalanceConnection> {
        Ok(txn_part::table
            .filter(txn_part::txn_id.eq(transaction_id))
            .inner_join(
                node::table
                    .inner_join(balance::table)
                    .on(txn_part::balance_id.eq(balance::id)),
            )
            .get_results_async::<(models::TransactionPart, models::Balance)>(pool)
            .await
            .map(|results| TransactionBalanceConnection {
                edges: results
                    .into_iter()
                    .map(|(transaction_part, balance)| TransactionBalanceEdge {
                        cursor: String::from(""),
                        node: balance.into(),
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
