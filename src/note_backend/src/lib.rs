#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell, sync::Mutex};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Note {
    id: u64,
    title: String,
    content: String,
    created_at: u64,
    updated_at: Option<u64>,
}

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

lazy_static::lazy_static! {
    static ref MEMORY_MANAGER: Mutex<MemoryManager<DefaultMemoryImpl>> =
        Mutex::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static ref ID_COUNTER: Mutex<IdCell> = {
        let memory_manager = MEMORY_MANAGER.lock().unwrap();
        Mutex::new(IdCell::init(memory_manager.get(MemoryId::new(0)), 0)
            .expect("Cannot create a counter"))
    };

    static ref NOTES_STORAGE: Mutex<StableBTreeMap<u64, Note, Memory>> =
        Mutex::new(StableBTreeMap::init(MEMORY_MANAGER.lock().unwrap().get(MemoryId::new(1))));
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct NotePayload {
    title: String,
    content: String,
}

#[ic_cdk::query]
fn get_note(id: u64) -> Result<Note, Error> {
    match _get_note(&id) {
        Some(note) => Ok(note),
        None => Err(Error::NotFound {
            msg: format!("A note with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn add_note(note_payload: NotePayload) -> Result<Note, Error> {
    let id = ID_COUNTER
        .lock()
        .unwrap()
        .update(|counter| counter + 1)
        .expect("Cannot increment id counter");

    let note = Note {
        id,
        title: note_payload.title,
        content: note_payload.content,
        created_at: time(),
        updated_at: None,
    };
    do_insert(&note);
    Ok(note)
}

#[ic_cdk::update]
fn update_note(id: u64, payload: NotePayload) -> Result<Note, Error> {
    let note = match NOTES_STORAGE.lock().unwrap().get(&id) {
        Some(mut note) => {
            note.content = payload.content;
            note.title = payload.title;
            note.updated_at = Some(time());
            note
        }
        None => {
            return Err(Error::NotFound {
                msg: format!("Couldn't update a note with id={}. Note not found", id),
            })
        }
    };

    do_insert(&note);
    Ok(note)
}

fn do_insert(note: &Note) {
    NOTES_STORAGE.lock().unwrap().insert(note.id, note.clone());
}

#[ic_cdk::update]
fn delete_note(id: u64) -> Result<Note, Error> {
    match NOTES_STORAGE.lock().unwrap().remove(&id) {
        Some(note) => Ok(note),
        None => Err(Error::NotFound {
            msg: format!("Couldn't delete a note with id={}. Note not found", id),
        }),
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

fn _get_note(id: &u64) -> Option<Note> {
    NOTES_STORAGE.lock().unwrap().get(id)
}

ic_cdk::export_candid!();
