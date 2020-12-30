mod cache;
pub mod googlesigninerror;

use crate::googlesignin::cache::{Cache, Certificates, HttpClient};
use crate::googlesignin::googlesigninerror::GoogleSignInError;
use hyper::client::Client as HyperClient;
use hyper_rustls::HttpsConnector;
use serde::Deserialize;

pub struct GoogleSignInClient {
    client: HttpClient,
    cache: Cache,
    pub audiences: Vec<String>,
    pub hosted_domains: Vec<String>,
}

impl GoogleSignInClient {
    pub fn new() -> GoogleSignInClient {
        let ssl = HttpsConnector::new();
        let client = HyperClient::builder()
            .http1_max_buf_size(0x2000)
            .pool_max_idle_per_host(0)
            .build(ssl);
        GoogleSignInClient {
            client,
            cache: Cache::new(),
            audiences: vec![],
            hosted_domains: vec![],
        }
    }

    /// Verifies that the token is signed by Google's OAuth cerificate,
    /// and check that it has a valid issuer, audience, and hosted domain.
    ///
    /// Returns an error if the client has no configured audiences.
    pub async fn verify(&self, id_token: &str) -> Result<IdInfo, GoogleSignInError> {
        let certs = self.cache.get_cached_or_refresh(&self.client).await?;
        self.verify_with(id_token, &certs).await
    }

    /// Verifies the token using the same method as `Client::verify`, but allows you to manually
    /// manage the lifetime of the certificates.
    ///
    /// This allows you to control when your application performs a network request (for example,
    /// to avoid network requests after dropping OS capabilities or outside of initialization).
    ///
    /// It is recommended to use `Client::verify` directly instead.
    pub async fn verify_with(
        &self,
        id_token: &str,
        cached_certs: &Certificates,
    ) -> Result<IdInfo, GoogleSignInError> {
        use jsonwebtoken::{Algorithm, DecodingKey, Validation};

        let unverified_header = jsonwebtoken::decode_header(&id_token)?;

        // Check each certificate
        for (_, cert) in cached_certs.get_range(&unverified_header.kid)? {
            let mut validation = Validation::new(Algorithm::RS256);
            validation.set_audience(&self.audiences);
            let token_data = jsonwebtoken::decode::<IdInfo>(
                &id_token,
                &DecodingKey::from_rsa_components(cert.get_n(), cert.get_e()),
                &validation,
            )?;

            let verification_result = token_data.claims.verify(self);
            if verification_result.is_ok() {
                return Ok(token_data.claims);
            }
        }

        Err(GoogleSignInError::InvalidToken)
    }
}

#[derive(Debug, Deserialize)]
pub struct IdInfo<EF = bool, TM = u64> {
    /// These six fields are included in all Google ID Tokens.
    pub iss: String,
    pub sub: String,
    pub azp: String,
    pub aud: String,
    pub iat: TM,
    pub exp: TM,

    /// This value indicates the user belongs to a Google Hosted Domain
    pub hd: Option<String>,

    /// These seven fields are only included when the user has granted the "profile" and
    /// "email" OAuth scopes to the application.
    pub email: Option<String>,
    pub email_verified: Option<EF>, // eg. "true" (but unusually as a string)
    pub name: Option<String>,
    pub picture: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub locale: Option<String>,
}

impl IdInfo {
    // Check the issuer, audiences, and (optionally) hosted domains of the IdInfo.
    //
    // Returns an error if the client has no configured audiences.
    fn verify(&self, client: &GoogleSignInClient) -> Result<(), GoogleSignInError> {
        // Check the id was authorized by google
        match self.iss.as_str() {
            "accounts.google.com" | "https://accounts.google.com" => {}
            _ => {
                return Err(GoogleSignInError::InvalidIssuer);
            }
        }

        // Check the token belongs to the application(s)
        if !client.audiences.is_empty() && !client.audiences.contains(&self.aud) {
            return Err(GoogleSignInError::InvalidAudience);
        }

        // Check the token belongs to the hosted domain(s)
        if !client.hosted_domains.is_empty() {
            match self.hd {
                Some(ref domain) if client.hosted_domains.contains(domain) => {}
                _ => {
                    return Err(GoogleSignInError::InvalidHostedDomain);
                }
            }
        }

        Ok(())
    }
}
