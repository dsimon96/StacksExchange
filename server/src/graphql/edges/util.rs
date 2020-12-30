use async_graphql::connection::CursorType;
use rmp_serde::{from_read_ref, to_vec};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_bytes;
use std::ops::Deref;
use thiserror::Error;

pub trait CursorDetail: Serialize + DeserializeOwned {}

#[derive(Debug)]
pub struct Cursor<T: CursorDetail>(pub T);

impl<T: CursorDetail> From<T> for Cursor<T> {
    fn from(cursor: T) -> Self {
        Cursor(cursor)
    }
}

impl<T: CursorDetail> Deref for Cursor<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize)]
pub enum CursorEncoding<'a> {
    Basic(#[serde(with = "serde_bytes")] &'a [u8]),
}

#[derive(Error, Debug)]
pub enum CustomCursorError {
    #[error("Failed to decode")]
    FailedToDecode(#[from] base64::DecodeError),
    #[error("Failed to deserialize")]
    FailedToDeserialize(#[from] rmp_serde::decode::Error),
}

impl<T: CursorDetail> CursorType for Cursor<T> {
    type Error = CustomCursorError;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        let decoded = base64::decode(&s)?;
        let ce: CursorEncoding = from_read_ref(&decoded)?;

        let serialized_cursor = match ce {
            CursorEncoding::Basic(buf) => buf,
        };

        Ok(Cursor(from_read_ref(&serialized_cursor)?))
    }

    fn encode_cursor(&self) -> String {
        let serialized_cursor = to_vec(&self.0).unwrap();
        let ce = CursorEncoding::Basic(&serialized_cursor);
        let decoded = to_vec(&ce).unwrap();
        base64::encode(&decoded)
    }
}
