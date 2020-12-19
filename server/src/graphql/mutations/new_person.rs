use crate::graphql::nodes::Person;
use async_graphql::validators::Email;

#[async_graphql::InputObject]
pub struct NewPersonInput {
    #[field(validator(Email))]
    pub email: String,
    pub display_name: String,
    pub first_name: String,
    pub last_name: String,
}

#[async_graphql::SimpleObject]
pub struct NewPersonPayload {
    pub person: Person,
}
