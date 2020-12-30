use super::{
    super::nodes::Transaction,
    util::{Cursor, CursorDetail, EdgeWithCursor, EdgesWrapper, NodeWrapper},
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

impl CursorDetail for BalanceTransactionCursorDetail {
    type NodeT = Transaction;

    fn get_for_node(node: &Self::NodeT) -> Self {
        Self::ByAscendingId(node.model.node.uid)
    }
}

pub type BalanceTransactionCursor = Cursor<BalanceTransactionCursorDetail>;

pub struct BalanceTransactionEdge {
    wrapper: NodeWrapper<BalanceTransactionCursorDetail>,
    balance_change_cents: i32,
}

#[async_graphql::Object]
impl BalanceTransactionEdge {
    pub async fn cursor(&self) -> String {
        self.wrapper.cursor().encode_cursor()
    }

    pub async fn node(&self) -> &Transaction {
        self.wrapper.node()
    }

    pub async fn balance_change_cents(&self) -> &i32 {
        &self.balance_change_cents
    }
}

impl EdgeWithCursor for BalanceTransactionEdge {
    type CursorT = BalanceTransactionCursor;

    fn get_cursor(&self) -> Self::CursorT {
        self.wrapper.cursor()
    }
}

pub struct BalanceTransactionConnection(EdgesWrapper<BalanceTransactionEdge>);

#[async_graphql::Object]
impl BalanceTransactionConnection {
    pub async fn edges(&self) -> &Vec<BalanceTransactionEdge> {
        self.0.edges()
    }

    pub async fn page_info(&self) -> PageInfo {
        self.0.page_info()
    }
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
                        wrapper: NodeWrapper(transaction.into()),
                        balance_change_cents: transaction_part.balance_change_cents,
                    })
                    .collect();

                Ok(BalanceTransactionConnection(EdgesWrapper {
                    edges,
                    has_previous_page: false,
                    has_next_page: false,
                }))
            },
        )
        .await
    }
}
