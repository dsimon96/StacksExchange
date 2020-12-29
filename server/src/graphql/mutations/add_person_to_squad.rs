use super::super::nodes::{Balance, Person, Squad};
use crate::db::{
    models,
    schema::{balance, node, person, squad},
    Pool,
};
use anyhow::Result;
use async_graphql::{FieldError, FieldResult, ID};
use diesel::prelude::*;
use std::convert::TryFrom;
use tokio_diesel::*;
use uuid::Uuid;

#[derive(async_graphql::InputObject)]
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
            Uuid::try_from(value.person_id).or_else(|_e| Err(FieldError::from("Invalid ID")))?;
        let squad_uid =
            Uuid::try_from(value.squad_id).or_else(|_e| Err(FieldError::from("Invalid ID")))?;

        Ok(ParsedAddPersonToSquadInput {
            person_uid,
            squad_uid,
        })
    }
}

#[derive(async_graphql::SimpleObject)]
pub struct AddPersonToSquadPayload {
    pub balance: Balance,
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

            let new_node = models::NewNode {
                uid: Uuid::new_v4(),
                node_type: models::NodeType::Balance,
            };

            let node = diesel::insert_into(node::table)
                .values(new_node)
                .get_result::<models::Node>(conn)?;

            let new_balance = models::NewBalance {
                node_id: node.id,
                person_id: person.detail.id,
                squad_id: squad.detail.id,
            };

            let balance = diesel::insert_into(balance::table)
                .values(&new_balance)
                .get_result::<models::BalanceDetail>(conn)
                .map(|detail| Balance {
                    model: models::Balance { node, detail },
                })?;

            Ok(AddPersonToSquadPayload {
                balance: balance.into(),
                person: person.into(),
                squad: squad.into(),
            })
        })
        .await?)
}
