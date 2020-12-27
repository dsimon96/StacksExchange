use super::{super::edges::TransactionBalanceConnection, Squad};
use crate::db::{models, Pool};
use async_graphql::{Context, FieldError, FieldResult};

pub struct Transaction {
    pub model: models::Transaction,
}

impl From<models::Transaction> for Transaction {
    fn from(model: models::Transaction) -> Self {
        Transaction { model }
    }
}

#[async_graphql::Object]
impl Transaction {
    pub async fn id(&self) -> String {
        self.model.node.uid.to_string()
    }

    pub async fn squad(&self, context: &Context<'_>) -> FieldResult<Squad> {
        Squad::by_id(context.data::<Pool>(), self.model.detail.squad_id)
            .await
            .or_else(|_e| Err(FieldError::from("Internal error")))
    }

    pub async fn balances(
        &self,
        context: &Context<'_>,
    ) -> FieldResult<TransactionBalanceConnection> {
        TransactionBalanceConnection::by_transaction_id(
            context.data::<Pool>(),
            self.model.detail.id,
        )
        .await
        .or_else(|_e| Err(FieldError::from("Internal error")))
    }
}
