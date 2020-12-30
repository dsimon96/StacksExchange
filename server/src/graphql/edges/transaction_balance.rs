use super::{
    super::nodes::Balance,
    util::{Cursor, CursorDetail, EdgeWithCursor, EdgesWrapper, NodeWrapper},
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

impl CursorDetail for TransactionBalanceCursorDetail {
    type NodeT = Balance;

    fn get_for_node(node: &Self::NodeT) -> Self {
        Self::ByAscendingId(node.model.node.uid)
    }
}

pub type TransactionBalanceCursor = Cursor<TransactionBalanceCursorDetail>;

pub struct TransactionBalanceEdge {
    wrapper: NodeWrapper<TransactionBalanceCursorDetail>,
    balance_change_cents: i32,
}

#[async_graphql::Object]
impl TransactionBalanceEdge {
    pub async fn cursor(&self) -> String {
        self.wrapper.cursor().encode_cursor()
    }

    pub async fn node(&self) -> &Balance {
        self.wrapper.node()
    }

    pub async fn balance_change_cents(&self) -> &i32 {
        &self.balance_change_cents
    }
}

impl EdgeWithCursor for TransactionBalanceEdge {
    type CursorT = TransactionBalanceCursor;

    fn get_cursor(&self) -> Self::CursorT {
        self.wrapper.cursor()
    }
}

pub struct TransactionBalanceConnection(EdgesWrapper<TransactionBalanceEdge>);

#[async_graphql::Object]
impl TransactionBalanceConnection {
    pub async fn edges(&self) -> &Vec<TransactionBalanceEdge> {
        self.0.edges()
    }

    pub async fn page_info(&self) -> PageInfo {
        self.0.page_info()
    }
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
                        wrapper: NodeWrapper(balance.into()),
                        balance_change_cents: transaction_part.balance_change_cents,
                    })
                    .collect();

                Ok(TransactionBalanceConnection(EdgesWrapper {
                    edges,
                    has_previous_page: false,
                    has_next_page: false,
                }))
            },
        )
        .await
    }
}
