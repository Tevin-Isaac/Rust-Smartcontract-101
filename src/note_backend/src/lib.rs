// Import necessary libraries
#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
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
    title: String,
    content: String,
    created_at: u64,
    updated_at: Option<u64>,
    tag_ids: Vec<u64>,
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
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Tag {
    id: u64,
    name: String,
}

impl Storable for Tag {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Tag {
    const MAX_SIZE: u32 = 1024; // Example size, adjust as needed
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct NoteVersion {
    id: u64,
    note_id: u64,
    title: String,
    content: String,
    version_number: u64,
    created_at: u64,
}

impl Storable for NoteVersion {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for NoteVersion {
    const MAX_SIZE: u32 = 1024; // Adjust as necessary
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
    static TAG_STORAGE: RefCell<StableBTreeMap<u64, Tag, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))) // Assuming MemoryId::new(2) is for tags
    ));
    static NOTE_VERSION_STORAGE: RefCell<StableBTreeMap<u64, NoteVersion, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))) // Assuming MemoryId::new(4) is for note versions
    ));
}

// Define the NotePyload structure for creating/updating notes
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct NotePayload {
    title: String,
    content: String,
}
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct TagPayload {
    name: String,
}
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct NoteVersionPayload {
    title: String,
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
    let notes: Vec<Note> = notes_map.into_iter().map(|(_, note)| note).collect();

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
        tag_ids: Vec::new(),
    };
    
    // Insert the note 
    do_insert(&note);
    Some(note)
}

// Function to update note
#[ic_cdk::update]
fn update_note(id: u64, payload: NotePayload) -> Result<Note, Error> {
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

// helper function to add note to storage
fn do_insert(note: &Note) {
    NOTES_STORAGE.with(|storage| storage.borrow_mut().insert(note.id, note.clone()));
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
#[ic_cdk::update]
fn add_tag(payload: TagPayload) -> Result<Tag, String> {
    TAG_STORAGE.with(|storage| {
        // Check if tag already exists
        {
            let storage_ref = storage.borrow(); // Immutable borrow
            if storage_ref.iter().any(|(_, tag)| tag.name == payload.name) {
                return Err("Tag with this name already exists".to_string());
            }
        } // Immutable borrow is dropped here

        // Add new tag
        let id = ID_COUNTER.with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1).unwrap();
            current_value
        });

        let tag = Tag { id, name: payload.name };
        storage.borrow_mut().insert(id, tag.clone()); // Mutable borrow
        Ok(tag)
    })
}


#[ic_cdk::query]
fn list_all_tags() -> Vec<Tag> {
    TAG_STORAGE.with(|storage| {
        storage.borrow().iter().map(|(_, tag)| tag.clone()).collect()
    })
}

#[ic_cdk::update]
fn update_tag(id: u64, payload: TagPayload) -> Result<Tag, String> {
    TAG_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        if let Some(tag) = storage.remove(&id) {
            let updated_tag = Tag { name: payload.name, ..tag };
            storage.insert(id, updated_tag.clone());
            Ok(updated_tag)
        } else {
            Err("Tag not found".to_string())
        }
    })
}
#[ic_cdk::update]
fn delete_tag(id: u64) -> Result<(), String> {
    TAG_STORAGE.with(|storage| {
        if storage.borrow_mut().remove(&id).is_some() {
            Ok(())
        } else {
            Err("Tag not found".to_string())
        }
    })
}
#[ic_cdk::update]
fn assign_tag_to_note(note_id: u64, tag_id: u64) -> Result<(), String> {
    NOTES_STORAGE.with(|notes| {
        let mut notes = notes.borrow_mut();
        if let Some(mut note) = notes.remove(&note_id) {
            if !note.tag_ids.contains(&tag_id) {
                note.tag_ids.push(tag_id);
                notes.insert(note_id, note);
                Ok(())
            } else {
                Err("Tag already assigned to this note".to_string())
            }
        } else {
            Err("Note not found".to_string())
        }
    })
}

#[ic_cdk::update]
fn remove_tag_from_note(note_id: u64, tag_id: u64) -> Result<(), String> {
    NOTES_STORAGE.with(|notes| {
        let mut notes = notes.borrow_mut();
        if let Some(mut note) = notes.remove(&note_id) {
            note.tag_ids.retain(|&id| id != tag_id);
            notes.insert(note_id, note);
            Ok(())
        } else {
            Err("Note not found".to_string())
        }
    })
}

#[ic_cdk::query]
fn get_notes_by_tag(tag_id: u64) -> Result<Vec<Note>, String> {
    NOTES_STORAGE.with(|notes| {
        let notes = notes.borrow();
        let filtered_notes = notes
            .iter()
            .filter(|(_, note)| note.tag_ids.contains(&tag_id))
            .map(|(_, note)| note.clone())
            .collect::<Vec<Note>>();
        
        if !filtered_notes.is_empty() {
            Ok(filtered_notes)
        } else {
            Err("No notes found for this tag".to_string())
        }
    })
}
#[ic_cdk::update]
fn create_note_version(note_id: u64, payload: NoteVersionPayload) -> Result<NoteVersion, String> {
    let version_number = NOTE_VERSION_STORAGE.with(|storage| {
        let versions = storage.borrow();
        versions.iter().filter(|(_, v)| v.note_id == note_id).count() as u64 + 1
    });

    let new_version = NoteVersion {
        id: ID_COUNTER.with(|c| { let val = *c.borrow().get(); c.borrow_mut().set(val + 1).unwrap(); val }),
        note_id,
        title: payload.title,
        content: payload.content,
        version_number,
        created_at: time(),
    };

    NOTE_VERSION_STORAGE.with(|storage| {
        storage.borrow_mut().insert(new_version.id, new_version.clone());
    });
    Ok(new_version)
}
#[ic_cdk::query]
fn get_note_version(version_id: u64) -> Result<NoteVersion, String> {
    NOTE_VERSION_STORAGE.with(|storage| {
        match storage.borrow().get(&version_id) {
            Some(note_version) => Ok(note_version.clone()),
            None => Err("Note version not found".to_string()),
        }
    })
}





#[ic_cdk::query]
fn list_note_versions(note_id: u64) -> Vec<NoteVersion> {
    NOTE_VERSION_STORAGE.with(|storage| {
        storage.borrow().iter().filter(|(_, v)| v.note_id == note_id).map(|(_, v)| v.clone()).collect()
    })
}
#[ic_cdk::update]
fn revert_to_version(note_id: u64, version_id: u64) -> Result<Note, String> {
    let version = get_note_version(version_id)?;
    if version.note_id != note_id {
        return Err("Version does not match the note".to_string());
    }

    NOTES_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        if let Some(mut note) = storage.remove(&note_id) {
            note.title = version.title;
            note.content = version.content;
            note.updated_at = Some(time());
            storage.insert(note_id, note.clone());
            Ok(note)
        } else {
            Err("Note not found".to_string())
        }
    })
}

#[ic_cdk::update]
fn delete_note_version(version_id: u64) -> Result<(), String> {
    NOTE_VERSION_STORAGE.with(|storage| {
        if storage.borrow_mut().remove(&version_id).is_some() {
            Ok(())
        } else {
            Err("Note version not found".to_string())
        }
    })
}



// Define the error enum for handling errors
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

// helper function to retrieve Note by ID
fn _get_note(id: &u64) -> Option<Note> {
    NOTES_STORAGE.with(|storage| storage.borrow().get(id))
}

// Export candid Interface
ic_cdk::export_candid!();