use super::{
    super::nodes::Transaction,
    util::{Cursor, CursorDetail, EdgeWithCursor, EdgesWrapper, NodeWrapper},
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

impl CursorDetail for SquadTransactionCursorDetail {
    type NodeT = Transaction;

    fn get_for_node(node: &Self::NodeT) -> Self {
        Self::ByAscendingId(node.model.node.uid)
    }
}

pub type SquadTransactionCursor = Cursor<SquadTransactionCursorDetail>;

pub struct SquadTransactionEdge(NodeWrapper<SquadTransactionCursorDetail>);

#[async_graphql::Object]
impl SquadTransactionEdge {
    pub async fn cursor(&self) -> String {
        self.0.cursor().encode_cursor()
    }

    pub async fn node(&self) -> &Transaction {
        self.0.node()
    }
}

impl EdgeWithCursor for SquadTransactionEdge {
    type CursorT = SquadTransactionCursor;

    fn get_cursor(&self) -> Self::CursorT {
        self.0.cursor()
    }
}

pub struct SquadTransactionConnection(EdgesWrapper<SquadTransactionEdge>);

#[async_graphql::Object]
impl SquadTransactionConnection {
    pub async fn edges(&self) -> &Vec<SquadTransactionEdge> {
        self.0.edges()
    }

    pub async fn page_info(&self) -> PageInfo {
        self.0.page_info()
    }
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
                    .map(|transaction| SquadTransactionEdge(NodeWrapper(transaction.into())))
                    .collect();

                Ok(SquadTransactionConnection(EdgesWrapper {
                    edges,
                    has_previous_page: false,
                    has_next_page: false,
                }))
            },
        )
        .await
    }
}
