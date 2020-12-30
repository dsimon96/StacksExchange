use super::{
    super::nodes::Balance,
    util::{Cursor, CursorDetail},
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

impl CursorDetail for SquadBalanceCursorDetail {}

pub type SquadBalanceCursor = Cursor<SquadBalanceCursorDetail>;

#[derive(async_graphql::SimpleObject)]
pub struct SquadBalanceEdge {
    pub cursor: String,
    pub node: Balance,
}

#[derive(async_graphql::SimpleObject)]
pub struct SquadBalanceConnection {
    pub edges: Vec<SquadBalanceEdge>,
    pub page_info: PageInfo,
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
                    .map(|balance| SquadBalanceEdge {
                        cursor: Cursor(SquadBalanceCursorDetail::ByAscendingId(balance.node.uid))
                            .encode_cursor(),
                        node: balance.into(),
                    })
                    .collect();

                let start_cursor = edges.first().map(|edge| edge.cursor.clone());
                let end_cursor = edges.last().map(|edge| edge.cursor.clone());

                Ok(SquadBalanceConnection {
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
