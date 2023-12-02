use crate::types::{Error, NotePayload};

pub fn validate_payload(payload: &NotePayload) -> Result<(), Error> {
    if payload.title.len() < 3 {
        return Err(Error::InvalidInput {
            msg: "Title should be more than 3 letters".to_string(),
        });
    }

    let word_count = payload.content.split_whitespace().count();
    if word_count < 5 {
        return Err(Error::InvalidInput {
            msg: "Content should be more than 5 words".to_string(),
        });
    }

    Ok(())
}
