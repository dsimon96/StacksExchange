use super::nodes::{Node, Person};
use crate::db::Pool;
use async_graphql::{validators::Email, Context, Result, ID};
use std::convert::TryFrom;
use uuid::Uuid;

/// Schema entry-point for queries
pub struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    pub async fn person_by_email(
        &self,
        context: &Context<'_>,
        #[graphql(validator(Email))] email: String,
    ) -> Result<Person> {
        Person::by_email(context.data::<Pool>().unwrap(), email).await
    }

    pub async fn node(&self, context: &Context<'_>, id: ID) -> Result<Node> {
        let uid = Uuid::try_from(id)?;

        Node::by_uid(context.data::<Pool>().unwrap(), uid).await
    }
}
