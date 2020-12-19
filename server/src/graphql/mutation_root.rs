use super::mutations::*;
use crate::db::Pool;
use async_graphql::{Context, FieldError, FieldResult};
use std::convert::TryInto;

/// Schema entry-point for mutations
pub struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    async fn new_person(
        &self,
        context: &Context<'_>,
        input: NewPersonInput,
    ) -> FieldResult<NewPersonPayload> {
        new_person(context.data::<Pool>(), input)
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
        new_squad(context.data::<Pool>(), input)
            .await
            .or_else(|_e| Err(FieldError::from("Failed to create new squad")))
    }

    async fn add_person_to_squad(
        &self,
        context: &Context<'_>,
        input: AddPersonToSquadInput,
    ) -> FieldResult<AddPersonToSquadPayload> {
        add_person_to_squad(context.data::<Pool>(), input.try_into()?)
            .await
            .or_else(|_e| Err(FieldError::from("Failed to add person to squad")))
    }
}
