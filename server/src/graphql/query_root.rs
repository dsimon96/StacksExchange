use crate::db::{self, models, schema::{node, person, squad}};
use async_graphql::{validators::Email, Context, FieldError, FieldResult, ID};
use crate::graphql::nodes::{Node, Person, Squad};
use uuid::Uuid;
use diesel::prelude::*;
use tokio_diesel::*;

/// Schema entry-point for queries
pub struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    pub async fn person_by_email(
        &self,
        context: &Context<'_>,
        #[arg(validator(Email))] email: String,
    ) -> FieldResult<Person> {
        node::table
            .inner_join(person::table)
            .filter(person::email.eq(email))
            .get_result_async::<models::Person>(context.data::<db::Pool>())
            .await
            .map(|person| person.into())
            .or_else(|_e| {
                Err(FieldError::from(
                    "Could not find a person with the given email",
                ))
            })
    }

    pub async fn node(&self, context: &Context<'_>, id: ID) -> FieldResult<Node> {
        let uid = Uuid::parse_str(&id).or_else(|_e| Err(FieldError::from("Invalid ID")))?;

        context
            .data::<db::Pool>()
            .transaction(move |conn| {
                let node = node::table
                    .filter(node::uid.eq(uid))
                    .get_result::<models::Node>(conn)?;

                match node.node_type {
                    models::NodeType::Person => {
                        let detail = person::table
                            .filter(person::node_id.eq(node.id))
                            .get_result::<models::PersonDetail>(conn)?;

                        Ok(Node::Person(Person {
                            model: models::Person { node, detail }.into(),
                        }))
                    }
                    models::NodeType::Squad => {
                        let detail = squad::table
                            .filter(squad::node_id.eq(node.id))
                            .get_result::<models::SquadDetail>(conn)?;

                        Ok(Node::Squad(Squad {
                            model: models::Squad { node, detail }.into(),
                        }))
                    }
                }
            })
            .await
            .or_else(|_e| Err(FieldError::from("Could not find a node with the given id")))
    }
}