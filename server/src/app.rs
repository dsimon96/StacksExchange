use actix_web::{get, post, web, Error, HttpResponse};
use crate::CONFIG;
use crate::graphql::{make_schema, Context, Schema};
use juniper::http::{graphiql::graphiql_source, GraphQLRequest};
use std::sync::Arc;

pub struct AppData {
    schema: Schema,
    context: Context,
}

impl AppData {
    pub fn new() -> AppData {
        AppData {
            schema: make_schema(),
            context: Context::new(),
        }
    }
}

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

#[get("/graphiql")]
async fn graphiql() -> HttpResponse {
    let addr = CONFIG.read().unwrap().get_str("addr").unwrap();
    let html = graphiql_source(&format!("http://{}/graphql", addr));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}
