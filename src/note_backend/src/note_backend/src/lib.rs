// Import necessary libraries
#[macro_use]
extern crate serde;
mod store;
mod types;
mod utils;
use ic_cdk::api::time;
use store::{_get_note, do_insert, ID_COUNTER, NOTES_STORAGE};
use types::{Error, Note, NotePayload};
use utils::validate_payload;

// Define the NotePyload structure for creating/updating notes

// Define a function to get a single note by ID
#[ic_cdk::query]
fn get_note(id: u64) -> Result<Note, Error> {
    match _get_note(&id) {
        Some(note) => Ok(note),
        None => Err(Error::NotFound {
            msg: format!("A note with id={} not found", id),
        }),
    }
}
// get all notes
#[ic_cdk::query]
fn get_all_notes() -> Result<Vec<Note>, Error> {
    let notes_map: Vec<(u64, Note)> =
        NOTES_STORAGE.with(|service| service.borrow().iter().collect());
    let notes: Vec<Note> = notes_map.into_iter().map(|(_, note)| note).collect();

    if !notes.is_empty() {
        Ok(notes)
    } else {
        Err(Error::NotFound {
            msg: "No notes found.".to_string(),
        })
    }
}

// Funcion to create a note
#[ic_cdk::update]
fn add_note(note_payload: NotePayload) -> Option<Note> {
    // Inrement the ID counter
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");
    // create a new note
    let note = Note {
        id,
        title: note_payload.title,
        content: note_payload.content,
        created_at: time(),
        updated_at: None,
    };

    // Insert the note
    do_insert(&note);
    Some(note)
}

// Function to update note
#[ic_cdk::update]
fn update_note(id: u64, payload: NotePayload) -> Result<Note, Error> {
    // Validate the payload
    validate_payload(&payload)?;

    match NOTES_STORAGE.with(|storage| storage.borrow().get(&id)) {
        Some(mut note) => {
            // update the note's content, title and timestamp
            note.content = payload.content;
            note.title = payload.title;
            note.updated_at = Some(time());
            do_insert(&note);
            Ok(note)
        }
        None => Err(Error::NotFound {
            msg: format!("Couldn't update a note with id={}. Note not found", id),
        }),
    }
}

// Function to delete note
#[ic_cdk::update]
fn delete_note(id: u64) -> Result<Note, Error> {
    match NOTES_STORAGE.with(|storage| storage.borrow_mut().remove(&id)) {
        Some(note) => Ok(note),
        None => Err(Error::NotFound {
            msg: format!("Couldn't delete a note with id={}. Note not found", id),
        }),
    }
}

// Export candid Interface
ic_cdk::export_candid!();
