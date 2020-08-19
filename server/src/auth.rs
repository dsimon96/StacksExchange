use crate::googlesignin::googlesigninerror::GoogleSignInError;
use crate::googlesignin::{GoogleSignInClient, IdInfo};
use actix_web::http::{Cookie, StatusCode};
use actix_web::web;
use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    credential: String,
    g_csrf_token: String,
}

pub async fn oauth_handler(req: HttpRequest, form: web::Form<FormData>) -> HttpResponse {
    // Verify double submit token to prevent CSRF.HttpMessage
    let cookie_o: Option<Cookie> = req.cookie("g_csrf_token");
    if cookie_o.is_none() {
        return HttpResponse::build(StatusCode::BAD_REQUEST).body("CSRF cookie not present.");
    }
    let cookie = cookie_o.unwrap();
    if form.g_csrf_token != cookie.value() {
        return HttpResponse::build(StatusCode::BAD_REQUEST)
            .body("CSRF - failed to verify double submit cookie.");
    }

    // Verify and exchange ID Token for IdInfo.
    let mut gsi_client = GoogleSignInClient::new();
    gsi_client.audiences.push(
        "962633347992-tbgvt8rcmnhdp5tlfm2hs1av8bkfc03n.apps.googleusercontent.com".to_string(),
    );
    let id_info: Result<IdInfo, GoogleSignInError> = gsi_client.verify(&form.credential).await;
    match id_info {
        Ok(s) => HttpResponse::Ok().body(format! {"IdInfo is {:?}", s}),
        Err(_) => HttpResponse::build(StatusCode::FAILED_DEPENDENCY).body("Token failed to verify"),
    }
}
