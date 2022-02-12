use super::super::{nodes::Balance, PageInfo};
use crate::db::{
    models,
    schema::{balance, node},
    Pool,
};
use async_graphql::Result;
use diesel::prelude::*;
use tokio_diesel::*;

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
    pub async fn by_person_id(pool: &Pool, person_id: i32) -> Result<PersonBalanceConnection> {
        Ok(node::table
            .inner_join(balance::table)
            .filter(balance::person_id.eq(person_id))
            .get_results_async::<models::Balance>(pool)
            .await
            .map(|results| PersonBalanceConnection {
                edges: results
                    .into_iter()
                    .map(|balance| PersonBalanceEdge {
                        cursor: String::from(""),
                        node: balance.into(),
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
