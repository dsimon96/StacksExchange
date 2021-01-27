use super::super::edges::PersonBalanceConnection;
use crate::db::{
    models,
    schema::{node, person},
    Pool,
};
use async_graphql::{Context, Result};
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

    pub async fn balances(&self, context: &Context<'_>) -> Result<PersonBalanceConnection> {
        PersonBalanceConnection::by_person_id(context.data::<Pool>().unwrap(), self.model.detail.id)
            .await
    }
}

impl Person {
    pub async fn by_email(pool: &Pool, email: String) -> Result<Person> {
        Ok(node::table
            .inner_join(person::table)
            .filter(person::email.eq(email))
            .get_result_async::<models::Person>(pool)
            .await
            .map(|person| person.into())?)
    }

    pub async fn by_id(pool: &Pool, id: i32) -> Result<Person> {
        Ok(node::table
            .inner_join(person::table)
            .filter(person::id.eq(id))
            .get_result_async::<models::Person>(pool)
            .await
            .map(|person| person.into())?)
    }
}
