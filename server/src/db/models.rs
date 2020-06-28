use super::schema::account;
use juniper::GraphQLObject;
use uuid::Uuid;

#[derive(GraphQLObject, Queryable, Identifiable)]
#[table_name = "account"]
pub struct Account {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Insertable)]
#[table_name = "account"]
pub struct NewAccount<'a> {
    pub id: &'a Uuid,
    pub email: &'a str,
    pub display_name: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
}
