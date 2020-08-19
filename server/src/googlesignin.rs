use bytes::buf::ext::BufExt;
use futures::future::{FutureExt, Shared};
use hyper::client::{Client as HyperClient, HttpConnector};
#[cfg(feature = "with-openssl")]
use hyper_openssl::HttpsConnector;
#[cfg(feature = "with-rustls")]
use hyper_rustls::HttpsConnector;
use std::collections::btree_map::Range;
use std::collections::BTreeMap;
use std::ops::{
    Bound,
    Bound::{Included, Unbounded},
};
use std::{fmt, io};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::Deserialize;

type HttpClient = HyperClient<HttpsConnector<HttpConnector>>;

pub struct Client {
    client: HttpClient,
    cache: Cache,
    pub audiences: Vec<String>,
    pub hosted_domains: Vec<String>,
}

impl Client {
    pub fn new() -> Client {
        #[cfg(feature = "with-rustls")]
        let ssl = HttpsConnector::new();
        #[cfg(feature = "with-openssl")]
        let ssl = HttpsConnector::new().expect("unable to build HttpsConnector");
        let client = HyperClient::builder()
            .http1_max_buf_size(0x2000)
            .keep_alive(false)
            .build(ssl);
        Client {
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
                &DecodingKey::from_rsa_components(&cert.n, &cert.e),
                &validation,
            )?;

            token_data.claims.verify(self)?;

            return Ok(token_data.claims);
        }

        Err(GoogleSignInError::InvalidToken)
    }

    /// Checks the token using Google's slow OAuth-like authentication flow.
    ///
    /// This checks that the token is signed using Google's OAuth certificate,
    /// but does not check the issuer, audience, or other application-specific verifications.
    ///
    /// This is NOT the recommended way to use the library, but can be used in combination with
    /// [IdInfo.verify](https://docs.rs/google-signin/latest/google_signin/struct.IdInfo.html#impl)
    /// for applications with more complex error-handling requirements.
    pub async fn get_slow_unverified(
        &self,
        id_token: &str,
    ) -> Result<IdInfo<String, String>, GoogleSignInError> {
        let url = format!(
            "https://www.googleapis.com/oauth2/v3/tokeninfo?id_token={}",
            id_token
        );
        let url = url.parse().unwrap();
        let response = self.client.get(url).await?;
        let status = response.status().as_u16();
        match status {
            200..=299 => {}
            _ => {
                return Err(GoogleSignInError::InvalidToken);
            }
        }
        let body = hyper::body::aggregate(response).await?;
        let data = serde_json::from_reader(body.reader())?;
        Ok(data)
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
    pub fn verify(&self, client: &Client) -> Result<(), GoogleSignInError> {
        // Check the id was authorized by google
        match self.iss.as_str() {
            "accounts.google.com" | "https://accounts.google.com" => {}
            _ => {
                return Err(GoogleSignInError::InvalidIssuer);
            }
        }

        // Check the token belongs to the application(s)
        if client.audiences.len() > 0 && !client.audiences.contains(&self.aud) {
            return Err(GoogleSignInError::InvalidAudience);
        }

        // Check the token belongs to the hosted domain(s)
        if client.hosted_domains.len() > 0 {
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

#[derive(Clone)]
struct Cache {
    state: Arc<Mutex<RefreshState>>,
}

impl Cache {
    fn new() -> Cache {
        Cache {
            state: Arc::new(Mutex::new(RefreshState::Uninitialized)),
        }
    }

    async fn get_cached_or_refresh(&self, client: &HttpClient) -> Result<Arc<Certificates>, GoogleSignInError> {
        // Acquire a lock in order to clone the Arc to the currently cached certificates,
        // or initialize a new future but don't block on it until after releasing the lock.
        let fut = {
            let mut guard = self.state.lock().unwrap();
            let state: &mut RefreshState = &mut guard;
            match state {
                RefreshState::Expired(fut) => fut.clone(),
                RefreshState::Uninitialized => {
                    let fut = Cache::refresh_with(self.state.clone(), client.clone())
                        .boxed_local()
                        .shared();
                    *state = RefreshState::Expired(fut.clone());
                    fut
                }
                RefreshState::Ready(certs) => {
                    if certs.is_expired() {
                        let fut = Cache::refresh_with(self.state.clone(), client.clone())
                            .boxed_local()
                            .shared();
                        *state = RefreshState::Expired(fut.clone());
                        fut
                    } else {
                        let certs = Arc::clone(certs);
                        (async move { Ok(certs) }).boxed_local().shared()
                    }
                }
            }
        };

        fut.await
    }

    async fn refresh_with(
        state: Arc<Mutex<RefreshState>>,
        client: HttpClient,
    ) -> Result<Arc<Certificates>, GoogleSignInError> {
        let certs = Certificates::get_with_http_client(&client).await?;
        let certs = Arc::new(certs);
        let mut state = state.lock().unwrap();
        *state = RefreshState::Ready(Arc::clone(&certs));
        Ok(certs)
    }
}

type Promise =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<Arc<Certificates>, GoogleSignInError>>>>;

enum RefreshState {
    Ready(Arc<Certificates>),
    Expired(Shared<Promise>),
    Uninitialized,
}

#[derive(Clone, Debug, Deserialize)]
struct CertsObject {
    keys: Vec<Cert>,
}

#[derive(Clone, Debug, Deserialize)]
struct Cert {
    kid: String,
    e: String,
    kty: String,
    alg: String,
    n: String,
    r#use: String,
}

type Key = String;

#[derive(Clone)]
pub struct Certificates {
    keys: BTreeMap<Key, Cert>,
    pub expiry: Option<Instant>,
}

impl Certificates {
    pub fn new() -> Self {
        Self {
            keys: BTreeMap::new(),
            expiry: None,
        }
    }

    /// Downloads the public Google certificates even if the current certificates have not expired.
    pub async fn get(client: &Client) -> Result<Certificates, GoogleSignInError> {
        Certificates::get_with_http_client(&client.client).await
    }

    async fn get_with_http_client(client: &HttpClient) -> Result<Certificates, GoogleSignInError> {
        const URL: &str = "https://www.googleapis.com/oauth2/v2/certs";

        let url = URL.parse().unwrap();
        let response = client.get(url).await?;
        let expiry = response
            .headers()
            .get("Cache-Control")
            .and_then(|val| val.to_str().ok())
            .and_then(cache_control::CacheControl::from_value)
            .and_then(|cc| cc.max_age)
            .and_then(|max_age| {
                let seconds = max_age.num_seconds();
                if seconds >= 0 {
                    Some(Instant::now() + Duration::from_secs(seconds as u64))
                } else {
                    None
                }
            });
        let body = hyper::body::aggregate(response).await?;
        let certs: CertsObject = serde_json::from_reader(body.reader())?;
        let mut keys = BTreeMap::new();
        for cert in certs.keys {
            keys.insert(cert.kid.clone(), cert);
        }
        Ok(Certificates { keys, expiry })
    }

    /// Downloads the public Google certificates if it didn't do so already, or based on expiry of
    /// their Cache-Control. Returns `true` if the certificates were updated.
    pub async fn refresh(&mut self) -> Result<bool, GoogleSignInError> {
        if !self.is_expired() {
            return Ok(false);
        }

        let client = Client::new();
        *self = Certificates::get(&client).await?;
        Ok(true)
    }

    /// Returns true if all cached certificates are expired (or if there are no cached certificates).
    pub fn is_expired(&self) -> bool {
        match self.expiry {
            Some(expiry) => expiry <= Instant::now(),
            None => true,
        }
    }

    fn get_range<'a>(&'a self, kid: &Option<String>) -> Result<Range<'a, Key, Cert>, GoogleSignInError> {
        match kid {
            None => Ok(self
                .keys
                .range::<String, (Bound<&String>, Bound<&String>)>((Unbounded, Unbounded))),
            Some(kid) => {
                if !self.keys.contains_key(kid) {
                    return Err(GoogleSignInError::InvalidKey);
                }
                Ok(self
                    .keys
                    .range::<String, (Bound<&String>, Bound<&String>)>((
                        Included(kid),
                        Included(kid),
                    )))
            }
        }
    }
}

/// A network or validation error
#[derive(Clone, Debug)]
pub enum GoogleSignInError {
    DecodeJson(Arc<serde_json::Error>),
    JSONWebToken(Arc<jsonwebtoken::errors::Error>),
    ConnectionError(Arc<dyn std::error::Error + Send + Sync + 'static>),
    InvalidKey,
    InvalidToken,
    InvalidIssuer,
    InvalidAudience,
    InvalidHostedDomain,
}

impl std::error::Error for GoogleSignInError {
    fn description(&self) -> &str {
        match *self {
            GoogleSignInError::DecodeJson(ref err) => err.description(),
            GoogleSignInError::ConnectionError(ref err) => err.description(),
            GoogleSignInError::JSONWebToken(ref err) => err.description(),
            GoogleSignInError::InvalidKey => "invalid key",
            GoogleSignInError::InvalidToken => "invalid token",
            GoogleSignInError::InvalidIssuer => "invalid issuer",
            GoogleSignInError::InvalidAudience => "invalid audience",
            GoogleSignInError::InvalidHostedDomain => "invalid hosted domain",
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            GoogleSignInError::DecodeJson(ref err) => Some(&**err),
            GoogleSignInError::ConnectionError(ref err) => Some(&**err),
            _ => None,
        }
    }
}

impl fmt::Display for GoogleSignInError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GoogleSignInError::DecodeJson(ref err) => err.fmt(f),
            GoogleSignInError::ConnectionError(ref err) => err.fmt(f),
            GoogleSignInError::JSONWebToken(ref err) => err.fmt(f),
            GoogleSignInError::InvalidKey => f.write_str("Token does not match any known key"),
            GoogleSignInError::InvalidToken => f.write_str("Token was not recognized by google"),
            GoogleSignInError::InvalidIssuer => f.write_str("Token was not issued by google"),
            GoogleSignInError::InvalidAudience => f.write_str("Token is for a different google application"),
            GoogleSignInError::InvalidHostedDomain => {
                f.write_str("User is not a member of the hosted domain(s)")
            }
        }
    }
}

impl From<io::Error> for GoogleSignInError {
    fn from(err: io::Error) -> GoogleSignInError {
        GoogleSignInError::ConnectionError(Arc::new(err))
    }
}

impl From<hyper::Error> for GoogleSignInError {
    fn from(err: hyper::Error) -> GoogleSignInError {
        GoogleSignInError::ConnectionError(Arc::new(err))
    }
}

impl From<serde_json::Error> for GoogleSignInError {
    fn from(err: serde_json::Error) -> GoogleSignInError {
        GoogleSignInError::DecodeJson(Arc::new(err))
    }
}

impl From<jsonwebtoken::errors::Error> for GoogleSignInError {
    fn from(err: jsonwebtoken::errors::Error) -> GoogleSignInError {
        GoogleSignInError::JSONWebToken(Arc::new(err))
    }
}
