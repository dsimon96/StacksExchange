use super::super::nodes::Person;
use crate::db::{
    models,
    schema::{node, person},
    Pool,
};
use async_graphql::validators::Email;
use diesel::prelude::*;
use tokio_diesel::*;
use uuid::Uuid;

#[async_graphql::InputObject]
pub struct NewPersonInput {
    #[field(validator(Email))]
    pub email: String,
    pub display_name: String,
    pub first_name: String,
    pub last_name: String,
}

#[async_graphql::SimpleObject]
pub struct NewPersonPayload {
    pub person: Person,
}

pub async fn new_person(pool: &Pool, input: NewPersonInput) -> AsyncResult<NewPersonPayload> {
    pool.transaction(move |conn| {
        let new_node = models::NewNode {
            uid: Uuid::new_v4(),
            node_type: models::NodeType::Person,
        };

        let node = diesel::insert_into(node::table)
            .values(new_node)
            .get_result::<models::Node>(conn)?;

        let new_person = models::NewPerson {
            node_id: node.id,
            email: &input.email,
            display_name: &input.display_name,
            first_name: &input.first_name,
            last_name: &input.last_name,
        };

        diesel::insert_into(person::table)
            .values(&new_person)
            .get_result::<models::PersonDetail>(conn)
            .map(|detail| NewPersonPayload {
                person: models::Person { node, detail }.into(),
            })
    })
    .await
}
