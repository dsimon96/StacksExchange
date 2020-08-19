use std::sync::Arc;
use std::{fmt, io};

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
            GoogleSignInError::DecodeJson(_) => "json decoding err",
            GoogleSignInError::ConnectionError(_) => "connection error",
            GoogleSignInError::JSONWebToken(_) => "JWT error",
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
            GoogleSignInError::InvalidAudience => {
                f.write_str("Token is for a different google application")
            }
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
