use super::super::nodes::{Person, Squad};
use crate::db::{
    models,
    schema::{node, person, person_squad_connection, squad},
    Pool,
};
use anyhow::Result;
use async_graphql::{FieldError, FieldResult, ID};
use diesel::prelude::*;
use std::convert::TryFrom;
use tokio_diesel::*;
use uuid::Uuid;

#[async_graphql::InputObject]
pub struct AddPersonToSquadInput {
    pub person_id: ID,
    pub squad_id: ID,
}

pub struct ParsedAddPersonToSquadInput {
    person_uid: Uuid,
    squad_uid: Uuid,
}

impl TryFrom<AddPersonToSquadInput> for ParsedAddPersonToSquadInput {
    type Error = FieldError;

    fn try_from(value: AddPersonToSquadInput) -> FieldResult<ParsedAddPersonToSquadInput> {
        let person_uid =
            Uuid::parse_str(&value.person_id).or_else(|_e| Err(FieldError::from("Invalid ID")))?;
        let squad_uid =
            Uuid::parse_str(&value.squad_id).or_else(|_e| Err(FieldError::from("Invalid ID")))?;

        Ok(ParsedAddPersonToSquadInput {
            person_uid,
            squad_uid,
        })
    }
}

#[async_graphql::SimpleObject]
pub struct AddPersonToSquadPayload {
    pub person: Person,
    pub squad: Squad,
}

pub async fn add_person_to_squad(
    pool: &Pool,
    input: ParsedAddPersonToSquadInput,
) -> Result<AddPersonToSquadPayload> {
    Ok(pool
        .transaction(move |conn| {
            let person = node::table
                .inner_join(person::table)
                .filter(node::uid.eq(input.person_uid))
                .get_result::<models::Person>(conn)?;

            let squad = node::table
                .inner_join(squad::table)
                .filter(node::uid.eq(input.squad_uid))
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
        .await?)
}
