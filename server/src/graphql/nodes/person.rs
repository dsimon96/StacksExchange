use super::super::{
    edges::{PersonSquadConnection, PersonSquadEdge},
    PageInfo,
};
use crate::db::{
    models,
    schema::{node, person, person_squad_connection, squad},
    Pool,
};
use async_graphql::{Context, FieldError, FieldResult};
use diesel::prelude::*;
use tokio_diesel::*;

pub struct Person {
    pub model: models::Person,
}

impl From<models::Person> for Person {
    fn from(model: models::Person) -> Self {
        Person { model }
    }
}

#[async_graphql::Object]
impl Person {
    pub async fn id(&self) -> String {
        self.model.node.uid.to_string()
    }

    pub async fn email(&self) -> &str {
        &self.model.detail.email
    }
    pub async fn display_name(&self) -> &str {
        &self.model.detail.display_name
    }
    pub async fn first_name(&self) -> &str {
        &self.model.detail.first_name
    }
    pub async fn last_name(&self) -> &str {
        &self.model.detail.last_name
    }

    pub async fn squads(&self, context: &Context<'_>) -> FieldResult<PersonSquadConnection> {
        use diesel::expression::dsl::any;

        let squad_ids = person_squad_connection::table
            .filter(person_squad_connection::person_id.eq(self.model.detail.id))
            .select(person_squad_connection::squad_id);

        node::table
            .inner_join(squad::table)
            .filter(squad::id.eq(any(squad_ids)))
            .load_async::<models::Squad>(context.data::<Pool>())
            .await
            .map(|squads| PersonSquadConnection {
                edges: squads
                    .into_iter()
                    .map(|squad| PersonSquadEdge {
                        cursor: String::from(""),
                        node: squad.into(),
                    })
                    .collect(),
                page_info: PageInfo {
                    has_next_page: false,
                    has_previous_page: false,
                    start_cursor: String::from(""),
                    end_cursor: String::from(""),
                },
            })
            .or_else(|_e| Err(FieldError::from("Internal error")))
    }
}

impl Person {
    pub async fn resolve_email(pool: &Pool, email: String) -> AsyncResult<Person> {
        node::table
            .inner_join(person::table)
            .filter(person::email.eq(email))
            .get_result_async::<models::Person>(pool)
            .await
            .map(|person| person.into())
    }
}
