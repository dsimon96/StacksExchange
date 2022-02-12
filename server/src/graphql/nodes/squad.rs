use super::super::edges::{SquadBalanceConnection, SquadTransactionConnection};
use crate::db::{
    models,
    schema::{node, squad},
    Pool,
};
use async_graphql::{Context, Result};
use diesel::prelude::*;
use tokio_diesel::*;

pub struct Squad {
    pub model: models::Squad,
}

impl From<models::Squad> for Squad {
    fn from(model: models::Squad) -> Self {
        Squad { model }
    }
}

#[async_graphql::Object]
impl Squad {
    pub async fn id(&self) -> String {
        self.model.node.uid.to_string()
    }

    pub async fn display_name(&self) -> &str {
        &self.model.detail.display_name
    }

    pub async fn balances(&self, context: &Context<'_>) -> Result<SquadBalanceConnection> {
        SquadBalanceConnection::by_squad_id(context.data::<Pool>().unwrap(), self.model.detail.id)
            .await
    }

    pub async fn transactions(&self, context: &Context<'_>) -> Result<SquadTransactionConnection> {
        SquadTransactionConnection::by_squad_id(
            context.data::<Pool>().unwrap(),
            self.model.detail.id,
        )
        .await
    }
}

impl Squad {
    pub async fn by_id(pool: &Pool, id: i32) -> Result<Squad> {
        Ok(node::table
            .inner_join(squad::table)
            .filter(squad::id.eq(id))
            .get_result_async::<models::Squad>(pool)
            .await
            .map(|squad| squad.into())?)
    }
}
