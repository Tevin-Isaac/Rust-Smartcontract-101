#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

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

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static NOTES_STORAGE: RefCell<StableBTreeMap<u64, Note, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
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
fn add_note(note_payload: NotePayload) -> Option<Note> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");
    let note = Note {
        id,
        title: note_payload.title,
        content: note_payload.content,
        created_at: time(),
        updated_at: None,
    };
    do_insert(&note);
    Some(note)
}

#[ic_cdk::update]
fn update_note(id: u64, payload: NotePayload) -> Result<Note, Error> {
    match NOTES_STORAGE.with(|storage| storage.borrow().get(&id)) {
        Some(mut note) => {
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

fn do_insert(note: &Note) {
    NOTES_STORAGE.with(|storage| storage.borrow_mut().insert(note.id, note.clone()));
}

#[ic_cdk::update]
fn delete_note(id: u64) -> Result<Note, Error> {
    match NOTES_STORAGE.with(|storage| storage.borrow_mut().remove(&id)) {
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
    NOTES_STORAGE.with(|storage| storage.borrow().get(id))
}

ic_cdk::export_candid!();
