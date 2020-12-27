use super::{Person, Squad};
use crate::db::{models, Pool};
use async_graphql::{Context, FieldError, FieldResult};

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

    pub async fn person(&self, context: &Context<'_>) -> FieldResult<Person> {
        Person::by_id(context.data::<Pool>(), self.model.detail.person_id)
            .await
            .or_else(|_e| Err(FieldError::from("Internal error")))
    }

    pub async fn squad(&self, context: &Context<'_>) -> FieldResult<Squad> {
        Squad::by_id(context.data::<Pool>(), self.model.detail.squad_id)
            .await
            .or_else(|_e| Err(FieldError::from("Internal error")))
    }
}
