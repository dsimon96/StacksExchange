use super::super::{nodes::Person, PageInfo};
use crate::db::{
    models,
    schema::{node, person, person_squad_connection},
    Pool,
};
use diesel::prelude::*;
use tokio_diesel::*;

#[async_graphql::SimpleObject]
pub struct SquadMemberEdge {
    pub cursor: String,
    pub node: Person,
    pub balance_cents: i32,
}

#[async_graphql::SimpleObject]
pub struct SquadMemberConnection {
    pub edges: Vec<SquadMemberEdge>,
    pub page_info: PageInfo,
}

impl SquadMemberConnection {
    pub async fn resolve_for_squad(
        pool: &Pool,
        squad_id: i32,
    ) -> AsyncResult<SquadMemberConnection> {
        pool.transaction(move |conn| {
            let detail = person_squad_connection::table
                .filter(person_squad_connection::squad_id.eq(squad_id));
            let people = node::table.inner_join(person::table);

            Ok(detail
                .inner_join(people.on(person_squad_connection::person_id.eq(person::id)))
                .get_results::<(models::PersonSquadConnection, models::Person)>(conn)?)
        })
        .await
        .map(|results| SquadMemberConnection {
            edges: results
                .into_iter()
                .map(|(connection, person)| SquadMemberEdge {
                    cursor: String::from(""),
                    node: person.into(),
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
