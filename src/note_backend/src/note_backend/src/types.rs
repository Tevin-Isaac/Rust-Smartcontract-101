use std::borrow::Cow;

use candid::{Decode, Encode};
use ic_stable_structures::{BoundedStorable, Storable};

// Define the Note structure
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
pub struct Note {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub created_at: u64,
    pub updated_at: Option<u64>,
}

// Implement Storable and BoundedStorable traits for Note
impl Storable for Note {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Note {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
pub struct NotePayload {
    pub title: String,
    pub content: String,
}

// Define the error enum for handling errors
#[derive(candid::CandidType, Deserialize, Serialize)]
pub enum Error {
    NotFound { msg: String },
    InvalidInput { msg: String },
}
