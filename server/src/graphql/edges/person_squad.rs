use super::super::{nodes::Squad, PageInfo};
use crate::db::{
    models,
    schema::{node, person_squad_connection, squad},
    Pool,
};
use diesel::prelude::*;
use tokio_diesel::*;

#[async_graphql::SimpleObject]
pub struct PersonSquadEdge {
    pub cursor: String,
    pub node: Squad,
    pub balance_cents: i32,
}

#[async_graphql::SimpleObject]
pub struct PersonSquadConnection {
    pub edges: Vec<PersonSquadEdge>,
    pub page_info: PageInfo,
}

impl PersonSquadConnection {
    pub async fn resolve_for_person(
        pool: &Pool,
        person_id: i32,
    ) -> AsyncResult<PersonSquadConnection> {
        pool.transaction(move |conn| {
            let detail = person_squad_connection::table
                .filter(person_squad_connection::person_id.eq(person_id));
            let squads = node::table.inner_join(squad::table);

            Ok(detail
                .inner_join(squads.on(person_squad_connection::squad_id.eq(squad::id)))
                .get_results::<(models::PersonSquadConnection, models::Squad)>(conn)?)
        })
        .await
        .map(|results| PersonSquadConnection {
            edges: results
                .into_iter()
                .map(|(connection, squad)| PersonSquadEdge {
                    cursor: String::from(""),
                    node: squad.into(),
                    balance_cents: connection.balance_cents,
                })
                .collect(),
            page_info: PageInfo {
                has_next_page: false,
                has_previous_page: false,
                start_cursor: String::from(""),
                end_cursor: String::from(""),
            },
        })
    }
}
