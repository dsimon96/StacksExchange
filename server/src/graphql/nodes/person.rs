use super::super::edges::PersonSquadConnection;
use crate::db::{
    models,
    schema::{node, person},
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
        PersonSquadConnection::resolve_for_person(context.data::<Pool>(), self.model.detail.id)
            .await
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
