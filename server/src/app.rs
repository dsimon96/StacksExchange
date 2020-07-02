use crate::db;
use crate::graphql::{Context, Schema};
use crate::settings::Settings;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use juniper::http::{graphiql::graphiql_source, playground::playground_source, GraphQLRequest};

/// Handler to execute a GraphQL request (either a query or a mutation)
pub async fn graphql(
    settings: web::Data<Settings>,
    schema: web::Data<Schema>,
    pool: web::Data<db::Pool>,
    req: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let context = Context::new(settings.get_ref().to_owned(), pool.get_ref().to_owned());
    let user = web::block(move || {
        let res = req.execute(&schema, &context);
        serde_json::to_string(&res)
    })
    .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(user))
}

/// Handler to provide graphiql for debuggability. Only exposed when compiled
/// with the 'graphiql' feature, to ensure that it is not exposed in prod
pub async fn graphiql(req: HttpRequest) -> HttpResponse {
    let url = req.url_for_static("graphql").unwrap();
    let html = graphiql_source(url.as_str());
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

/// Handler to provide graphql playground for query development. Only exposed
/// when compiled with the 'graphiql' feature, to ensure that it is not exposed
/// in prod
pub async fn playground(req: HttpRequest) -> HttpResponse {
    let url = req.url_for_static("graphql").unwrap();
    let html = playground_source(url.as_str());
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    #[actix_rt::test]
    async fn test_graphiql_and_playground() {
        let mut app = test::init_service(
            App::new()
                .service(
                    web::resource("graphql")
                        .name("graphql")
                        .route(web::post().to(graphql))
                        .route(web::get().to(graphql)),
                )
                .route("graphiql", web::get().to(graphiql))
                .route("playground", web::get().to(playground)),
        )
        .await;

        let req = test::TestRequest::get().uri("/graphiql").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(
            resp.status().is_success(),
            "Failed to fetch graphiql: {:?}",
            resp
        );

        let req = test::TestRequest::get().uri("/playground").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(
            resp.status().is_success(),
            "Failed to fetch playground: {:?}",
            resp
        );
    }
}
