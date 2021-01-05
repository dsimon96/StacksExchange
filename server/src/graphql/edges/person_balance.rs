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
pub enum PersonBalanceCursorDetail {
    ByAscendingId(Uuid),
}

impl CursorDetail for PersonBalanceCursorDetail {
    type NodeT = Balance;

    fn get_for_node(node: &Self::NodeT) -> Self {
        Self::ByAscendingId(node.model.node.uid)
    }
}

pub type PersonBalanceCursor = Cursor<PersonBalanceCursorDetail>;

pub struct PersonBalanceEdge(NodeWrapper<PersonBalanceCursorDetail>);

#[async_graphql::Object]
impl PersonBalanceEdge {
    pub async fn cursor(&self) -> String {
        self.0.cursor().encode_cursor()
    }

    pub async fn node(&self) -> &Balance {
        self.0.node()
    }
}

impl EdgeWithCursor for PersonBalanceEdge {
    type CursorT = PersonBalanceCursor;

    fn get_cursor(&self) -> Self::CursorT {
        self.0.cursor()
    }
}

pub struct PersonBalanceConnection(EdgesWrapper<PersonBalanceEdge>);

#[async_graphql::Object]
impl PersonBalanceConnection {
    pub async fn edges(&self) -> &Vec<PersonBalanceEdge> {
        self.0.edges()
    }

    pub async fn page_info(&self) -> PageInfo {
        self.0.page_info()
    }
}

impl PersonBalanceConnection {
    pub async fn by_person_id(
        pool: &Pool,
        person_id: i32,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> Result<PersonBalanceConnection> {
        query_with(
            after,
            before,
            first,
            last,
            |_after: Option<PersonBalanceCursor>,
             _before: Option<PersonBalanceCursor>,
             _first,
             _last| async move {
                let balances = node::table
                    .inner_join(balance::table)
                    .filter(balance::person_id.eq(person_id))
                    .order(balance::id.asc())
                    .get_results_async::<models::Balance>(pool)
                    .await?;

                let edges: Vec<PersonBalanceEdge> = balances
                    .into_iter()
                    .map(|balance| PersonBalanceEdge(NodeWrapper(balance.into())))
                    .collect();

                Ok(PersonBalanceConnection(EdgesWrapper {
                    edges,
                    has_previous_page: false,
                    has_next_page: false,
                }))
            },
        )
        .await
    }
}
