pub mod edges;
pub mod mutations;
pub mod nodes;

mod mutation_root;
mod page_info;
mod query_root;

pub use mutation_root::*;
pub use page_info::*;
pub use query_root::*;

use crate::{db, settings::Settings};
use async_graphql::EmptySubscription;

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
