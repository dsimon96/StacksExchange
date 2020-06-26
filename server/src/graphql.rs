use juniper::FieldResult;
use juniper::{GraphQLEnum, GraphQLInputObject, GraphQLObject};
use juniper::RootNode;

#[derive(GraphQLEnum)]
enum Episode {
    NewHope,
    Empire,
    Jedi,
}

#[derive(GraphQLObject)]
#[graphql(description = "A humanoid creature in the Star Wars universe")]
struct Human {
    id: String,
    name: String,
    appears_in: Vec<Episode>,
    home_planet: String,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "A humanoid creature in the Star Wars universe")]
struct NewHuman {
    name: String,
    appears_in: Vec<Episode>,
    home_planet: String,
}

/// Schema entry-point for queries
pub struct QueryRoot;

#[juniper::object(
    Context = Context,
)]
impl QueryRoot {
    fn human(id: String) -> FieldResult<Human> {
        Ok(Human {
            id: "1234".to_owned(),
            name: "Luke".to_owned(),
            appears_in: vec![Episode::NewHope],
            home_planet: "Mars".to_owned(),
        })
    }
}

/// Schema entry-point for mutations
pub struct MutationRoot;

#[juniper::object(
    Context = Context
)]
impl MutationRoot {
    fn createHuman(new_human: NewHuman) -> FieldResult<Human> {
        Ok(Human {
            id: "1234".to_owned(),
            name: new_human.name,
            appears_in: new_human.appears_in,
            home_planet: new_human.home_planet,
        })
    }
}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn make_schema() -> Schema {
    RootNode::new(QueryRoot {}, MutationRoot {})
}

/// State shared across queries
pub struct Context {}

impl Context {
    pub fn new() -> Context {
        Context {}
    }
}