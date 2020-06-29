use crate::db::{
    self, models,
    schema::{node, person},
};
use crate::settings::Settings;
use diesel::prelude::*;
use fast_chemail::is_valid_email;
use juniper::{
    graphql_interface, graphql_value, FieldError, FieldResult, GraphQLInputObject, GraphQLObject,
    RootNode, ID,
};
use std::time::Duration;
use uuid::Uuid;

#[derive(GraphQLObject)]
struct Person {
    id: ID,
    email: String,
    display_name: String,
    first_name: String,
    last_name: String,
}

impl From<models::Person> for Person {
    fn from(person: models::Person) -> Self {
        Person {
            id: ID::new(person.node.uid.to_string()),
            email: person.detail.email,
            display_name: person.detail.display_name,
            first_name: person.detail.first_name,
            last_name: person.detail.last_name,
        }
    }
}

enum Node {
    Person(Person),
}

graphql_interface!(Node: () where Scalar = <S> |&self| {
    field id() -> ID {
        match *self {
            Node::Person(Person { ref id, .. }) => id.clone()
        }
    }

    instance_resolvers: |_| {
        &Person => match *self { Node::Person(ref p) => Some(p) }
    }
});

#[derive(GraphQLInputObject)]
struct NewPersonInput {
    email: String,
    display_name: String,
    first_name: String,
    last_name: String,
}

/// Schema entry-point for queries
pub struct QueryRoot;

#[juniper::object(
    Context = Context,
)]
impl QueryRoot {
    fn person_by_email(context: &Context, email: String) -> FieldResult<Person> {
        if !is_valid_email(&email) {
            return Err(FieldError::new(
                "Invalid email address",
                graphql_value!(None),
            ));
        }

        let pool_timeout = Duration::from_millis(context.settings.db.pool_timeout_ms);
        let conn = context.pool.get_timeout(pool_timeout)?;

        node::table
            .inner_join(person::table)
            .filter(person::email.eq(email))
            .get_result::<models::Person>(&conn)
            .map(|person| person.into())
            .or_else(|e| {
                Err(FieldError::new(
                    "Could not find a person with the given email",
                    graphql_value!(None),
                ))
            })
    }

    fn node(context: &Context, id: ID) -> FieldResult<Node> {
        let acct_id = Uuid::parse_str(&id.to_string()).or(Err("Invalid id"))?;

        let pool_timeout = Duration::from_millis(context.settings.db.pool_timeout_ms);
        let conn = context.pool.get_timeout(pool_timeout)?;

        node::table
            .inner_join(person::table)
            .filter(node::uid.eq(&acct_id))
            .get_result::<models::Person>(&conn)
            .map(|person| Node::Person(person.into()))
            .or_else(|e| {
                Err(FieldError::new(
                    "Could not find a node with the given id",
                    graphql_value!(None),
                ))
            })
    }
}

/// Schema entry-point for mutations
pub struct MutationRoot;

#[juniper::object(
    Context = Context,
)]
impl MutationRoot {
    fn newPerson(context: &Context, input: NewPersonInput) -> FieldResult<Person> {
        if !is_valid_email(&input.email) {
            return Err(FieldError::new(
                "Invalid email address",
                graphql_value!(None),
            ));
        }

        let pool_timeout = Duration::from_millis(context.settings.db.pool_timeout_ms);
        let conn = context.pool.get_timeout(pool_timeout)?;

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
            .or_else(|e| {
                // TODO: provide feedback on duplicate email or display_name
                Err(FieldError::new(
                    "Failed to create new account",
                    graphql_value!(None),
                ))
            })
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn make_schema() -> Schema {
    RootNode::new(QueryRoot {}, MutationRoot {})
}

/// State shared across queries
pub struct Context {
    settings: Settings,
    pool: db::Pool,
}

impl Context {
    pub fn new(settings: Settings, pool: db::Pool) -> Context {
        Context { settings, pool }
    }
}
