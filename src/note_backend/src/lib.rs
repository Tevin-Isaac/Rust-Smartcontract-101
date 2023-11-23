// Import necessary libraries
#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use validator::Validate;
use ic_cdk::api::{time, caller};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

// Define Struct and types
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Define the Note structure 
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Note {
    id: u64,
    owner: String,
    title: String,
    content: String,
    created_at: u64,
    updated_at: Option<u64>,
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

// Define Memory manager and ID counter
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

// Define the NotePyload structure for creating/updating notes
#[derive(candid::CandidType, Serialize, Deserialize, Default, Validate)]
struct NotePayload {
    #[validate(length(min = 1))]
    title: String,
    #[validate(length(min = 1))]
    content: String,
}

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
fn get_all_notes() -> Result<Vec<Note>, Error>{
    let notes_map: Vec<(u64,Note)> = NOTES_STORAGE.with(|service| service.borrow().iter().collect());
    let notes: Vec<Note> = notes_map.into_iter().map(|(_, crop)| crop).collect();

    if !notes.is_empty(){
        Ok(notes)
    }else {
        Err(Error::NotFound{
            msg: "No notes found.".to_string(),
        })
    }
}

// Funcion to create a note
#[ic_cdk::update]
fn add_note(note_payload: NotePayload) -> Result<Note, Error> {
    // Validates payload
    let check_payload = _check_input(&note_payload);
    // Returns an error if validations failed
    if check_payload.is_err(){
        return Err(check_payload.err().unwrap());
    }
    // Increment the ID counter
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment id counter");
    // create a new note 
    let note = Note {
        id,
        owner: caller().to_string(),
        title: note_payload.title,
        content: note_payload.content,
        created_at: time(),
        updated_at: None,
    };
    
    // Insert the note 
    do_insert(&note);
    Ok(note)
}

// Function to update note
#[ic_cdk::update]
fn update_note(id: u64, payload: NotePayload) -> Result<Note, Error> {
    // Validates payload
    let check_payload = _check_input(&payload);
    // Returns an error if validations failed
    if check_payload.is_err(){
        return Err(check_payload.err().unwrap());
    }
    match NOTES_STORAGE.with(|storage| storage.borrow().get(&id)) {
        Some(mut note) => {
            // Validates whether caller is the owner of the note
            let check_if_owner = _check_if_owner(&note);
            if check_if_owner.is_err() {
                return Err(check_if_owner.err().unwrap())
            }
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

// helper function to add note to storage
fn do_insert(note: &Note) {
    NOTES_STORAGE.with(|storage| storage.borrow_mut().insert(note.id, note.clone()));
}

// Function to delete note 
#[ic_cdk::update]
fn delete_note(id: u64) -> Result<Note, Error> {
    let note = _get_note(&id).expect(&format!("couldn't delete a note with id={}. note not found.", id));
    // Validates whether caller is the owner of the note
    let check_if_owner = _check_if_owner(&note);
    if check_if_owner.is_err() {
        return Err(check_if_owner.err().unwrap())
    }
    match NOTES_STORAGE.with(|storage| storage.borrow_mut().remove(&id)) {
        Some(note) => Ok(note),
        None => Err(Error::NotFound {
            msg: format!("Couldn't delete a note with id={}. Note not found", id),
        }),
    }
}

// Define the error enum for handling errors
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    ValidationFailed { content: String},
    AuthenticationFailed { msg: String}
}

// helper function to retrieve Note by ID
fn _get_note(id: &u64) -> Option<Note> {
    NOTES_STORAGE.with(|storage| storage.borrow().get(id))
}

// Helper function to check the input data of the payload
fn _check_input(payload: &NotePayload) -> Result<(), Error> {
    let check_payload = payload.validate();
    if check_payload.is_err() {
        return Err(Error:: ValidationFailed{ content: check_payload.err().unwrap().to_string()})
    }else{
        Ok(())
    }
}

// Helper function to check whether the caller is the owner of a note
fn _check_if_owner(note: &Note) -> Result<(), Error> {
    if note.owner.to_string() != caller().to_string(){
        return Err(Error:: AuthenticationFailed{ msg: format!("Caller={} isn't the owner of the note with id={}", caller(), note.id) })  
    }else{
        Ok(())
    }
}

// Export candid Interface
ic_cdk::export_candid!();
