use crate::googlesignin::googlesigninerror::GoogleSignInError;
use bytes::buf::ext::BufExt;
use futures::future::{FutureExt, Shared};
use hyper::client::{Client as HyperClient, HttpConnector};
#[cfg(feature = "with-openssl")]
use hyper_openssl::HttpsConnector;
#[cfg(feature = "with-rustls")]
use hyper_rustls::HttpsConnector;
use serde::Deserialize;
use std::collections::btree_map::Range;
use std::collections::BTreeMap;
use std::ops::{
    Bound,
    Bound::{Included, Unbounded},
};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub type HttpClient = HyperClient<HttpsConnector<HttpConnector>>;

#[derive(Clone)]
pub struct Cache {
    state: Arc<Mutex<RefreshState>>,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            state: Arc::new(Mutex::new(RefreshState::Uninitialized)),
        }
    }

    pub async fn get_cached_or_refresh(
        &self,
        client: &HttpClient,
    ) -> Result<Arc<Certificates>, GoogleSignInError> {
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

type Promise = std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<Arc<Certificates>, GoogleSignInError>>>,
>;

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
pub struct Cert {
    kid: String,
    e: String,
    kty: String,
    alg: String,
    n: String,
    r#use: String,
}

impl Cert {
    pub fn get_n(&self) -> &str {
        &self.n
    }

    pub fn get_e(&self) -> &str {
        &self.e
    }
}

type Key = String;

#[derive(Clone)]
pub struct Certificates {
    keys: BTreeMap<Key, Cert>,
    pub expiry: Option<Instant>,
}

impl Certificates {
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

    /// Returns true if all cached certificates are expired (or if there are no cached certificates).
    pub fn is_expired(&self) -> bool {
        match self.expiry {
            Some(expiry) => expiry <= Instant::now(),
            None => true,
        }
    }

    pub fn get_range<'a>(
        &'a self,
        kid: &Option<String>,
    ) -> Result<Range<'a, Key, Cert>, GoogleSignInError> {
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
