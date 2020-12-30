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
pub enum PersonBalanceCursorDetail {
    ByAscendingId(Uuid),
}

impl CursorDetail for PersonBalanceCursorDetail {}

pub type PersonBalanceCursor = Cursor<PersonBalanceCursorDetail>;

#[derive(async_graphql::SimpleObject)]
pub struct PersonBalanceEdge {
    pub cursor: String,
    pub node: Balance,
}

#[derive(async_graphql::SimpleObject)]
pub struct PersonBalanceConnection {
    pub edges: Vec<PersonBalanceEdge>,
    pub page_info: PageInfo,
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
                    .map(|balance| PersonBalanceEdge {
                        cursor: Cursor(PersonBalanceCursorDetail::ByAscendingId(balance.node.uid))
                            .encode_cursor(),
                        node: balance.into(),
                    })
                    .collect();

                let start_cursor = edges.first().map(|edge| edge.cursor.clone());
                let end_cursor = edges.last().map(|edge| edge.cursor.clone());

                Ok(PersonBalanceConnection {
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
