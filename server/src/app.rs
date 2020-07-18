use crate::graphql::Schema;
use actix_web::{web, HttpResponse};
use async_graphql::http::{graphiql_source, playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{GQLRequest, GQLResponse};

/// Handler to execute a GraphQL request (either a query or a mutation)
pub async fn graphql(schema: web::Data<Schema>, req: GQLRequest) -> GQLResponse {
    req.into_inner().execute(&schema).await.into()
}

/// Handler to provide graphiql for debuggability. Only exposed when compiled
/// with the 'graphiql' feature, to ensure that it is not exposed in prod
pub async fn graphiql() -> HttpResponse {
    let html = graphiql_source("/graphql", None);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

/// Handler to provide graphql playground for query development. Only exposed
/// when compiled with the 'graphiql' feature, to ensure that it is not exposed
/// in prod
pub async fn playground() -> HttpResponse {
    let html = playground_source(GraphQLPlaygroundConfig::new("/graphql"));
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
