use crate::db::{
    self,
    models::{Account, NewAccount},
    schema::account,
};
use crate::settings::Settings;
use diesel::prelude::*;
use fast_chemail::is_valid_email;
use juniper::{graphql_value, FieldError, FieldResult, GraphQLInputObject, RootNode};
use std::time::Duration;
use uuid::Uuid;

#[derive(GraphQLInputObject)]
struct CreateAccountInput {
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
    fn accountById(context: &Context, id: String) -> FieldResult<Account> {
        let acct_id = Uuid::parse_str(&id).or(Err("Invalid id"))?;

        let pool_timeout = Duration::from_millis(context.settings.db.pool_timeout_ms);
        let conn = context.pool.get_timeout(pool_timeout)?;

        account::table
            .find(&acct_id)
            .get_result(&*conn)
            .or_else(|e| {
                Err(FieldError::new(
                    "Could not find an account with the given id",
                    graphql_value!(None),
                ))
            })
    }

    fn accountByEmail(context: &Context, email: String) -> FieldResult<Account> {
        if !is_valid_email(&email) {
            return Err(FieldError::new(
                "Invalid email address",
                graphql_value!(None),
            ));
        }

        let pool_timeout = Duration::from_millis(context.settings.db.pool_timeout_ms);
        let conn = context.pool.get_timeout(pool_timeout)?;

        account::table
            .filter(account::email.eq(email))
            .get_result(&*conn)
            .or_else(|e| {
                Err(FieldError::new(
                    "Could not find an account with the given email",
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
    fn createAccount(context: &Context, input: CreateAccountInput) -> FieldResult<Account> {
        if !is_valid_email(&input.email) {
            return Err(FieldError::new(
                "Invalid email address",
                graphql_value!(None),
            ));
        }

        let new_account = NewAccount {
            id: &Uuid::new_v4(),
            email: &input.email,
            display_name: &input.display_name,
            first_name: &input.first_name,
            last_name: &input.last_name,
        };

        let pool_timeout = Duration::from_millis(context.settings.db.pool_timeout_ms);
        let conn = context.pool.get_timeout(pool_timeout)?;

        diesel::insert_into(account::table)
            .values(&new_account)
            .get_result(&*conn)
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
