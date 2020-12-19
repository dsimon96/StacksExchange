mod person;
mod squad;

pub use person::*;
pub use squad::*;

#[async_graphql::Interface(field(name = "id", type = "String"))]
pub enum Node {
    Person(Person),
    Squad(Squad),
}