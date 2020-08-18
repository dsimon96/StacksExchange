use actix_web::HttpResponse;

pub async fn oauth_handler() -> HttpResponse {
    HttpResponse::Ok().body("Hello world!")
}