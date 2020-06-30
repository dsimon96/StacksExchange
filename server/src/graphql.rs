use crate::db::{
    self, models,
    schema::{node, person},
};
use crate::settings::Settings;
use async_graphql::{validators::Email, Context, EmptySubscription, FieldError, FieldResult, ID};
use diesel::prelude::*;
use std::time::Duration;
use uuid::Uuid;

struct Person {
    id: ID,
    email: String,
    display_name: String,
    first_name: String,
    last_name: String,
}

#[async_graphql::Object]
impl Person {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn email(&self) -> &str {
        &self.email
    }
    async fn display_name(&self) -> &str {
        &self.display_name
    }
    async fn first_name(&self) -> &str {
        &self.first_name
    }
    async fn last_name(&self) -> &str {
        &self.last_name
    }
}

impl From<models::Person> for Person {
    fn from(person: models::Person) -> Self {
        Person {
            id: person.node.uid.into(),
            email: person.detail.email,
            display_name: person.detail.display_name,
            first_name: person.detail.first_name,
            last_name: person.detail.last_name,
        }
    }
}

#[async_graphql::Interface(field(name = "id", type = "&str"))]
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

/// Schema entry-point for queries
pub struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    async fn person_by_email(
        &self,
        context: &Context<'_>,
        #[arg(validator(Email))] email: String,
    ) -> FieldResult<Person> {
        let pool_timeout = Duration::from_millis(context.data::<Settings>().db.pool_timeout_ms);
        let conn = context.data::<db::Pool>().get_timeout(pool_timeout)?;

        node::table
            .inner_join(person::table)
            .filter(person::email.eq(email))
            .get_result::<models::Person>(&conn)
            .map(|person| person.into())
            .or_else(|_e| {
                Err(FieldError::from(
                    "Could not find a person with the given email",
                ))
            })
    }

    async fn node(&self, context: &Context<'_>, id: ID) -> FieldResult<Node> {
        let acct_id = Uuid::parse_str(&id).or(Err(FieldError::from("Invalid ID")))?;

        let pool_timeout = Duration::from_millis(context.data::<Settings>().db.pool_timeout_ms);
        let conn = context.data::<db::Pool>().get_timeout(pool_timeout)?;

        node::table
            .inner_join(person::table)
            .filter(node::uid.eq(&acct_id))
            .get_result::<models::Person>(&conn)
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
    ) -> FieldResult<Person> {
        let pool_timeout = Duration::from_millis(context.data::<Settings>().db.pool_timeout_ms);
        let conn = context.data::<db::Pool>().get_timeout(pool_timeout)?;

        conn.build_transaction()
            .run(|| {
                let node = diesel::insert_into(node::table)
                    .values(node::uid.eq(Uuid::new_v4()))
                    .returning((node::id, node::uid))
                    .get_result::<models::Node>(&conn)?;

                let new_person = models::NewPerson {
                    node_id: node.id,
                    email: &input.email,
                    display_name: &input.display_name,
                    first_name: &input.first_name,
                    last_name: &input.last_name,
                };

                diesel::insert_into(person::table)
                    .values(&new_person)
                    .get_result::<models::PersonDetail>(&conn)
                    .map(|detail| models::Person { node, detail }.into())
            })
            .or_else(|_e| {
                // TODO: provide feedback on duplicate email or display_name
                Err(FieldError::from("Failed to create new account"))
            })
    }
}

pub type Schema = async_graphql::Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn make_schema(settings: Settings, pool: db::Pool) -> Schema {
    Schema::build(QueryRoot {}, MutationRoot {}, EmptySubscription {})
        .data(settings)
        .data(pool)
        .finish()
}
