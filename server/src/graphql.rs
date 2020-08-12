use crate::db::{
    self, models,
    schema::{node, person, person_squad_connection, squad},
};
use crate::settings::Settings;
use async_graphql::{validators::Email, Context, EmptySubscription, FieldError, FieldResult, ID};
use diesel::prelude::*;
use log;
use tokio_diesel::*;
use uuid::Uuid;

struct Person {
    model: models::Person,
}

struct Squad {
    model: models::Squad,
}

#[async_graphql::SimpleObject]
struct PageInfo {
    has_next_page: bool,
    has_previous_page: bool,
    start_cursor: String,
    end_cursor: String,
}

#[async_graphql::SimpleObject]
struct PersonSquadEdge {
    cursor: String,
    node: Squad,
}

#[async_graphql::SimpleObject]
struct PersonSquadConnection {
    edges: Vec<PersonSquadEdge>,
    page_info: PageInfo,
}

#[async_graphql::SimpleObject]
struct SquadMemberEdge {
    cursor: String,
    node: Person,
}

#[async_graphql::SimpleObject]
struct SquadMemberConnection {
    edges: Vec<SquadMemberEdge>,
    page_info: PageInfo,
}

#[async_graphql::Object]
impl Person {
    async fn id(&self) -> String {
        self.model.node.uid.to_string()
    }

    async fn email(&self) -> &str {
        &self.model.detail.email
    }
    async fn display_name(&self) -> &str {
        &self.model.detail.display_name
    }
    async fn first_name(&self) -> &str {
        &self.model.detail.first_name
    }
    async fn last_name(&self) -> &str {
        &self.model.detail.last_name
    }

    async fn squads(&self, context: &Context<'_>) -> FieldResult<PersonSquadConnection> {
        use diesel::expression::dsl::any;

        let squad_ids = person_squad_connection::table
            .filter(person_squad_connection::person_id.eq(self.model.detail.id))
            .select(person_squad_connection::squad_id);

        node::table
            .inner_join(squad::table)
            .filter(squad::id.eq(any(squad_ids)))
            .load_async::<models::Squad>(context.data::<db::Pool>())
            .await
            .map(|squads| PersonSquadConnection {
                edges: squads
                    .into_iter()
                    .map(|squad| PersonSquadEdge {
                        cursor: String::from(""),
                        node: squad.into(),
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

#[async_graphql::Object]
impl Squad {
    async fn id(&self) -> String {
        self.model.node.uid.to_string()
    }

    async fn display_name(&self) -> &str {
        &self.model.detail.display_name
    }

    async fn members(&self, context: &Context<'_>) -> FieldResult<SquadMemberConnection> {
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

impl From<models::Person> for Person {
    fn from(model: models::Person) -> Self {
        Person { model }
    }
}

impl From<models::Squad> for Squad {
    fn from(model: models::Squad) -> Self {
        Squad { model }
    }
}

#[async_graphql::Interface(field(name = "id", type = "String"))]
enum Node {
    Person(Person),
}

#[async_graphql::InputObject]
struct NewPersonInput {
    #[field(validator(Email))]
    email: String,
    display_name: String,
    first_name: String,
    last_name: String,
}

#[async_graphql::SimpleObject]
struct NewPersonPayload {
    person: Person,
}

#[async_graphql::InputObject]
struct NewSquadInput {
    display_name: String,
}

#[async_graphql::SimpleObject]
struct NewSquadPayload {
    squad: Squad,
}

#[async_graphql::InputObject]
struct AddPersonToSquadInput {
    person_id: ID,
    squad_id: ID,
}

#[async_graphql::SimpleObject]
struct AddPersonToSquadPayload {
    person: Person,
    squad: Squad,
}

/// Schema entry-point for queries
pub struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    async fn person_by_email(
        &self,
        context: &Context<'_>,
        #[arg(validator(Email))] email: String,
    ) -> FieldResult<Person> {
        node::table
            .inner_join(person::table)
            .filter(person::email.eq(email))
            .get_result_async::<models::Person>(context.data::<db::Pool>())
            .await
            .map(|person| person.into())
            .or_else(|_e| {
                Err(FieldError::from(
                    "Could not find a person with the given email",
                ))
            })
    }

    async fn node(&self, context: &Context<'_>, id: ID) -> FieldResult<Node> {
        let acct_id = Uuid::parse_str(&id).or_else(|_e| Err(FieldError::from("Invalid ID")))?;

        node::table
            .inner_join(person::table)
            .filter(node::uid.eq(acct_id))
            .get_result_async::<models::Person>(context.data::<db::Pool>())
            .await
            .map(|person| Node::Person(person.into()))
            .or_else(|_e| Err(FieldError::from("Could not find a node with the given id")))
    }
}

/// Schema entry-point for mutations
pub struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    async fn new_person(
        &self,
        context: &Context<'_>,
        input: NewPersonInput,
    ) -> FieldResult<NewPersonPayload> {
        context
            .data::<db::Pool>()
            .transaction(move |conn| {
                let node = diesel::insert_into(node::table)
                    .values(node::uid.eq(Uuid::new_v4()))
                    .returning((node::id, node::uid))
                    .get_result::<models::Node>(conn)?;

                let new_person = models::NewPerson {
                    node_id: node.id,
                    email: &input.email,
                    display_name: &input.display_name,
                    first_name: &input.first_name,
                    last_name: &input.last_name,
                };

                diesel::insert_into(person::table)
                    .values(&new_person)
                    .get_result::<models::PersonDetail>(conn)
                    .map(|detail| NewPersonPayload {
                        person: models::Person { node, detail }.into(),
                    })
            })
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
        context
            .data::<db::Pool>()
            .transaction(move |conn| {
                let node = diesel::insert_into(node::table)
                    .values(node::uid.eq(Uuid::new_v4()))
                    .returning((node::id, node::uid))
                    .get_result::<models::Node>(conn)?;

                let new_squad = models::NewSquad {
                    node_id: node.id,
                    display_name: &input.display_name,
                };

                diesel::insert_into(squad::table)
                    .values(&new_squad)
                    .get_result::<models::SquadDetail>(conn)
                    .map(|detail| NewSquadPayload {
                        squad: models::Squad { node, detail }.into(),
                    })
            })
            .await
            .or_else(|_e| {
                // TODO: provide feedback on duplicate display_name
                Err(FieldError::from("Failed to create new squad"))
            })
    }

    async fn add_person_to_squad(
        &self,
        context: &Context<'_>,
        input: AddPersonToSquadInput,
    ) -> FieldResult<AddPersonToSquadPayload> {
        let person_uid =
            Uuid::parse_str(&input.person_id).or_else(|_e| Err(FieldError::from("Invalid ID")))?;
        let squad_uid =
            Uuid::parse_str(&input.squad_id).or_else(|_e| Err(FieldError::from("Invalid ID")))?;

        context
            .data::<db::Pool>()
            .transaction(move |conn| {
                let person = node::table
                    .inner_join(person::table)
                    .filter(node::uid.eq(person_uid))
                    .get_result::<models::Person>(conn)?;

                let squad = node::table
                    .inner_join(squad::table)
                    .filter(node::uid.eq(squad_uid))
                    .get_result::<models::Squad>(conn)?;

                log::info!(
                    "Node id: {}, Squad id: {}",
                    squad.detail.node_id,
                    squad.detail.id
                );

                let new_person_squad_connection = models::NewPersonSquadConnection {
                    person_id: person.detail.id,
                    squad_id: squad.detail.id,
                };

                diesel::insert_into(person_squad_connection::table)
                    .values(&new_person_squad_connection)
                    .execute(conn)?;

                Ok(AddPersonToSquadPayload {
                    person: person.into(),
                    squad: squad.into(),
                })
            })
            .await
            .or_else(|_e| {
                // TODO: provide feedback on duplicate display_name
                Err(FieldError::from("Failed to add person to squad"))
            })
    }
}

pub type Schema = async_graphql::Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn make_schema(settings: Settings, pool: db::Pool) -> Schema {
    let mut builder = Schema::build(QueryRoot {}, MutationRoot {}, EmptySubscription {})
        .data(settings)
        .data(pool)
        .extension(|| async_graphql::extensions::Logger::default());

    #[cfg(feature = "graphiql")]
    {
        builder = builder.extension(|| async_graphql::extensions::ApolloTracing::default());
    }

    #[cfg(not(feature = "graphiql"))]
    {
        builder = builder.disable_introspection();
    }

    builder.finish()
}
