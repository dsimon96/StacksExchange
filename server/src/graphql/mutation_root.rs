use super::mutations::*;
use crate::db::Pool;
use async_graphql::{Context, Result};
use std::convert::TryInto;

/// Schema entry-point for mutations
pub struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    async fn new_person(
        &self,
        context: &Context<'_>,
        input: NewPersonInput,
    ) -> Result<NewPersonPayload> {
        new_person(context.data::<Pool>().unwrap(), input).await
    }

    async fn new_squad(
        &self,
        context: &Context<'_>,
        input: NewSquadInput,
    ) -> Result<NewSquadPayload> {
        new_squad(context.data::<Pool>().unwrap(), input).await
    }

    async fn add_person_to_squad(
        &self,
        context: &Context<'_>,
        input: AddPersonToSquadInput,
    ) -> Result<AddPersonToSquadPayload> {
        add_person_to_squad(context.data::<Pool>().unwrap(), input.try_into()?).await
    }

    async fn new_transaction(
        &self,
        context: &Context<'_>,
        input: NewTransactionInput,
    ) -> Result<NewTransactionPayload> {
        new_transaction(context.data::<Pool>().unwrap(), input.try_into()?).await
    }
}
