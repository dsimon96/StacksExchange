use crate::db::{self, models, schema::{node, person, squad, person_squad_connection}};
use async_graphql::{Context, FieldError, FieldResult};
use crate::graphql::{mutations::{AddPersonToSquadInput, AddPersonToSquadPayload, NewPersonInput, NewPersonPayload, NewSquadInput, NewSquadPayload}};
use uuid::Uuid;
use diesel::prelude::*;
use tokio_diesel::*;

/// Schema entry-point for mutations
pub struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    async fn new_person(
        &self,
        context: &Context<'_>,
        input: NewPersonInput,
    ) -> FieldResult<NewPersonPayload> {
        context
            .data::<db::Pool>()
            .transaction(move |conn| {
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
            .or_else(|_e| {
                // TODO: provide feedback on duplicate email or display_name
                Err(FieldError::from("Failed to create new account"))
            })
    }

    async fn new_squad(
        &self,
        context: &Context<'_>,
        input: NewSquadInput,
    ) -> FieldResult<NewSquadPayload> {
        context
            .data::<db::Pool>()
            .transaction(move |conn| {
                let new_node = models::NewNode {
                    uid: Uuid::new_v4(),
                    node_type: models::NodeType::Squad,
                };

                let node = diesel::insert_into(node::table)
                    .values(new_node)
                    .get_result::<models::Node>(conn)?;

                let new_squad = models::NewSquad {
                    node_id: node.id,
                    display_name: &input.display_name,
                };

                diesel::insert_into(squad::table)
                    .values(&new_squad)
                    .get_result::<models::SquadDetail>(conn)
                    .map(|detail| NewSquadPayload {
                        squad: models::Squad { node, detail }.into(),
                    })
            })
            .await
            .or_else(|_e| Err(FieldError::from("Failed to create new squad")))
    }

    async fn add_person_to_squad(
        &self,
        context: &Context<'_>,
        input: AddPersonToSquadInput,
    ) -> FieldResult<AddPersonToSquadPayload> {
        let person_uid =
            Uuid::parse_str(&input.person_id).or_else(|_e| Err(FieldError::from("Invalid ID")))?;
        let squad_uid =
            Uuid::parse_str(&input.squad_id).or_else(|_e| Err(FieldError::from("Invalid ID")))?;

        context
            .data::<db::Pool>()
            .transaction(move |conn| {
                let person = node::table
                    .inner_join(person::table)
                    .filter(node::uid.eq(person_uid))
                    .get_result::<models::Person>(conn)?;

                let squad = node::table
                    .inner_join(squad::table)
                    .filter(node::uid.eq(squad_uid))
                    .get_result::<models::Squad>(conn)?;

                let new_person_squad_connection = models::NewPersonSquadConnection {
                    person_id: person.detail.id,
                    squad_id: squad.detail.id,
                };

                diesel::insert_into(person_squad_connection::table)
                    .values(&new_person_squad_connection)
                    .execute(conn)?;

                Ok(AddPersonToSquadPayload {
                    person: person.into(),
                    squad: squad.into(),
                })
            })
            .await
            .or_else(|_e| Err(FieldError::from("Failed to add person to squad")))
    }
}
