use super::super::edges::SquadBalanceConnection;
use crate::db::{
    models,
    schema::{node, squad},
    Pool,
};
use async_graphql::{Context, FieldError, FieldResult};
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

    pub async fn balances(&self, context: &Context<'_>) -> FieldResult<SquadBalanceConnection> {
        SquadBalanceConnection::by_squad_id(context.data::<Pool>(), self.model.detail.id)
            .await
            .or_else(|_e| Err(FieldError::from("Internal error")))
    }
}

impl Squad {
    pub async fn by_id(pool: &Pool, id: i32) -> AsyncResult<Squad> {
        node::table
            .inner_join(squad::table)
            .filter(squad::id.eq(id))
            .get_result_async::<models::Squad>(pool)
            .await
            .map(|squad| squad.into())
    }
}
