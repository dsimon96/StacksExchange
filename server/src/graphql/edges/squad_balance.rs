use super::{
    super::nodes::Balance,
    util::{Cursor, CursorDetail, EdgeWithCursor, EdgesWrapper, NodeWrapper},
};
use crate::db::{
    models,
    schema::{balance, node},
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
pub enum SquadBalanceCursorDetail {
    ByAscendingId(Uuid),
}

impl CursorDetail for SquadBalanceCursorDetail {
    type NodeT = Balance;

    fn get_for_node(node: &Self::NodeT) -> Self {
        Self::ByAscendingId(node.model.node.uid)
    }
}

pub type SquadBalanceCursor = Cursor<SquadBalanceCursorDetail>;

pub struct SquadBalanceEdge(NodeWrapper<SquadBalanceCursorDetail>);

#[async_graphql::Object]
impl SquadBalanceEdge {
    pub async fn cursor(&self) -> String {
        self.0.cursor().encode_cursor()
    }

    pub async fn node(&self) -> &Balance {
        self.0.node()
    }
}

impl EdgeWithCursor for SquadBalanceEdge {
    type CursorT = SquadBalanceCursor;

    fn get_cursor(&self) -> Self::CursorT {
        self.0.cursor()
    }
}

pub struct SquadBalanceConnection(EdgesWrapper<SquadBalanceEdge>);

#[async_graphql::Object]
impl SquadBalanceConnection {
    pub async fn edges(&self) -> &Vec<SquadBalanceEdge> {
        self.0.edges()
    }

    pub async fn page_info(&self) -> PageInfo {
        self.0.page_info()
    }
}

impl SquadBalanceConnection {
    pub async fn by_squad_id(
        pool: &Pool,
        squad_id: i32,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<SquadBalanceConnection> {
        query_with(
            after,
            before,
            first,
            last,
            |_after: Option<SquadBalanceCursor>,
             _before: Option<SquadBalanceCursor>,
             _first,
             _last| async move {
                let balances = node::table
                    .inner_join(balance::table)
                    .filter(balance::squad_id.eq(squad_id))
                    .order(balance::id.asc())
                    .get_results_async::<models::Balance>(pool)
                    .await?;

                let edges: Vec<SquadBalanceEdge> = balances
                    .into_iter()
                    .map(|balance| SquadBalanceEdge(NodeWrapper(balance.into())))
                    .collect();

                Ok(SquadBalanceConnection(EdgesWrapper {
                    edges,
                    has_previous_page: false,
                    has_next_page: false,
                }))
            },
        )
        .await
    }
}
