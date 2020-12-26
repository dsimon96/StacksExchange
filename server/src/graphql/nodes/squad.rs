use super::super::edges::SquadMemberConnection;
use crate::db::{models, Pool};
use async_graphql::{Context, FieldError, FieldResult};

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

    pub async fn members(&self, context: &Context<'_>) -> FieldResult<SquadMemberConnection> {
        SquadMemberConnection::resolve_for_squad(context.data::<Pool>(), self.model.detail.id)
            .await
            .or_else(|_e| Err(FieldError::from("Internal error")))
    }
}
