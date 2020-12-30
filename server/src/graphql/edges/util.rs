use async_graphql::{
    connection::{CursorType, PageInfo},
    OutputType,
};
use rmp_serde::{from_read_ref, to_vec};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::ops::Deref;
use thiserror::Error;

pub trait CursorDetail: Serialize + DeserializeOwned {
    type NodeT: OutputType + Send + Sync;

    fn get_for_node(node: &Self::NodeT) -> Self;
}

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

pub struct NodeWrapper<T: CursorDetail>(pub T::NodeT);

impl<T: CursorDetail> NodeWrapper<T> {
    pub fn cursor(&self) -> Cursor<T> {
        Cursor(T::get_for_node(&self.0))
    }

    pub fn node(&self) -> &T::NodeT {
        &self.0
    }
}

pub trait EdgeWithCursor: OutputType + Send + Sync {
    type CursorT: CursorType;

    fn get_cursor(&self) -> Self::CursorT;
}

pub struct EdgesWrapper<EdgeT: EdgeWithCursor> {
    pub edges: Vec<EdgeT>,
    pub has_previous_page: bool,
    pub has_next_page: bool,
}

impl<T: EdgeWithCursor> EdgesWrapper<T> {
    pub fn edges(&self) -> &Vec<T> {
        &self.edges
    }

    pub fn page_info(&self) -> PageInfo {
        PageInfo {
            has_previous_page: self.has_previous_page,
            has_next_page: self.has_next_page,
            start_cursor: self
                .edges
                .first()
                .map(|edge| edge.get_cursor().encode_cursor()),
            end_cursor: self
                .edges
                .last()
                .map(|edge| edge.get_cursor().encode_cursor()),
        }
    }
}
