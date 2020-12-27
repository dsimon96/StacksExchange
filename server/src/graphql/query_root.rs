use super::nodes::{Node, Person};
use crate::db::Pool;
use async_graphql::{validators::Email, Context, FieldError, FieldResult, ID};
use uuid::Uuid;

/// Schema entry-point for queries
pub struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    pub async fn person_by_email(
        &self,
        context: &Context<'_>,
        #[arg(validator(Email))] email: String,
    ) -> FieldResult<Person> {
        Person::by_email(context.data::<Pool>(), email)
            .await
            .or_else(|_e| {
                Err(FieldError::from(
                    "Could not find a person with the given email",
                ))
            })
    }

    pub async fn node(&self, context: &Context<'_>, id: ID) -> FieldResult<Node> {
        let uid = Uuid::parse_str(&id).or_else(|_e| Err(FieldError::from("Invalid ID")))?;

        Node::by_uid(context.data::<Pool>(), uid)
            .await
            .or_else(|_e| Err(FieldError::from("Could not find a node with the given id")))
    }
}
