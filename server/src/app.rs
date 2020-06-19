use actix_web::{get, post, web, Error, HttpResponse};
use config::Config;
use crate::graphql::{make_schema, Context, Schema};
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};
use std::sync::Arc;

/// State store shared across request invocations. Only available to requests
/// as an immutable reference, so use interior mutability for mutable state
pub struct AppData {
    schema: Schema,
    context: Context,
    config: Config
}

impl AppData {
    pub fn new(config: Config) -> AppData {
        AppData {
            schema: make_schema(),
            context: Context::new(),
            config
        }
    }
}

/// Handler to execute a GraphQL request (either a query or a mutation)
#[post("/graphql")]
async fn graphql(
    data: web::Data<Arc<AppData>>,
    req: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let user = web::block(move || {
        let res = req.execute(&data.schema, &data.context);
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(user))
}

/// Handler to provide graphiql for debuggability. Only exposed when compiled
/// with the 'graphiql' feature, to ensure that it is not exposed in prod
#[get("/graphiql")]
async fn graphiql(
    data: web::Data<Arc<AppData>>
) -> HttpResponse {
    let addr = data.config.get_str("addr").unwrap();
    let html = graphiql_source(&format!("http://{}/graphql", addr));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}
