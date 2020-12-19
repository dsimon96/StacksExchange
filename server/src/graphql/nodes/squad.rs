use crate::db::{self, models, schema::{node, person, person_squad_connection}};
use async_graphql::{Context, FieldError, FieldResult};
use crate::graphql::{PageInfo, edges::{SquadMemberConnection, SquadMemberEdge}};
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

    pub async fn members(&self, context: &Context<'_>) -> FieldResult<SquadMemberConnection> {
        use diesel::expression::dsl::any;

        let person_ids = person_squad_connection::table
            .filter(person_squad_connection::squad_id.eq(self.model.detail.id))
            .select(person_squad_connection::person_id);

        node::table
            .inner_join(person::table)
            .filter(person::id.eq(any(person_ids)))
            .load_async::<models::Person>(context.data::<db::Pool>())
            .await
            .map(|people| SquadMemberConnection {
                edges: people
                    .into_iter()
                    .map(|person| SquadMemberEdge {
                        cursor: String::from(""),
                        node: person.into(),
                    })
                    .collect(),
                page_info: PageInfo {
                    has_next_page: false,
                    has_previous_page: false,
                    start_cursor: String::from(""),
                    end_cursor: String::from(""),
                },
            })
            .or_else(|_e| Err(FieldError::from("Internal error")))
    }
}
