use crate::googlesignin::{Client, GoogleSignInError, IdInfo};
use actix_web::web;
use actix_web::HttpResponse;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    credential: String,
    g_csrf_token: String,
}

/// Extract form data using serde.
/// This handler get called only if content type is *x-www-form-urlencoded*
/// and content of the request could be deserialized to a `FormData` struct
pub async fn oauth_handler(form: web::Form<FormData>) -> HttpResponse {
    let b: String = format!(
        "credential is {} and crsf_token is {}!",
        form.credential, form.g_csrf_token
    );
    let mut client = Client::new();
    client.audiences.push(
        "962633347992-tbgvt8rcmnhdp5tlfm2hs1av8bkfc03n.apps.googleusercontent.com".to_string(),
    );
    let id_info: Result<IdInfo, GoogleSignInError> = client.verify(&form.credential).await;
    match id_info {
        Ok(s) => HttpResponse::Ok().body(format! {"IdInfo is {:?}", s}),
        Err(e) => HttpResponse::InternalServerError().finish(),
    }
}
