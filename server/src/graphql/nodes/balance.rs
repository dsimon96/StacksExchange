use super::super::edges::BalanceTransactionConnection;
use super::{Person, Squad};
use crate::db::{models, schema::txn_part, Pool};
use async_graphql::{Context, FieldError, FieldResult};
use diesel::prelude::*;
use std::convert::TryFrom;
use tokio_diesel::*;

pub struct Balance {
    pub model: models::Balance,
}

impl From<models::Balance> for Balance {
    fn from(model: models::Balance) -> Self {
        Balance { model }
    }
}

#[async_graphql::Object]
impl Balance {
    pub async fn id(&self) -> String {
        self.model.node.uid.to_string()
    }

    pub async fn total_cents(&self, context: &Context<'_>) -> FieldResult<i32> {
        use diesel::dsl::sum;

        let sum = txn_part::table
            .filter(txn_part::balance_id.eq(self.model.detail.id))
            .select(sum(txn_part::balance_change_cents))
            .get_result_async::<Option<i64>>(context.data::<Pool>().unwrap())
            .await?;

        Ok(sum
            .map(|n| i32::try_from(n).expect("Exceeded maximum representable balance"))
            .unwrap_or(0))
    }

    pub async fn person(&self, context: &Context<'_>) -> FieldResult<Person> {
        Person::by_id(context.data::<Pool>().unwrap(), self.model.detail.person_id)
            .await
            .or_else(|_e| Err(FieldError::from("Internal error")))
    }

    pub async fn squad(&self, context: &Context<'_>) -> FieldResult<Squad> {
        Squad::by_id(context.data::<Pool>().unwrap(), self.model.detail.squad_id)
            .await
            .or_else(|_e| Err(FieldError::from("Internal error")))
    }

    pub async fn transactions(
        &self,
        context: &Context<'_>,
    ) -> FieldResult<BalanceTransactionConnection> {
        BalanceTransactionConnection::by_balance_id(context.data::<Pool>().unwrap(), self.model.detail.id)
            .await
            .or_else(|_e| Err(FieldError::from("Internal error")))
    }
}
