extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, sync::{Arc, Mutex}};

type Memory = VirtualMemory<DefaultMemoryImpl>;

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
        Cow::Owned(Encode!(self).expect("Serialization failed"))
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Deserialization failed")
    }
}

impl BoundedStorable for Note {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct NotePayload {
    title: String,
    content: String,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

struct NoteService {
    memory_manager: MemoryManager<DefaultMemoryImpl>,
    id_counter: Arc<Mutex<u64>>,
    notes_storage: StableBTreeMap<u64, Note, Memory>,
}

impl NoteService {
    fn new() -> Self {
        let memory_manager = MemoryManager::init(DefaultMemoryImpl::default());
        let id_counter = Arc::new(Mutex::new(0));
        let notes_storage = StableBTreeMap::init(memory_manager.get(MemoryId::new(1)));
        
        NoteService {
            memory_manager,
            id_counter,
            notes_storage,
        }
    }

    fn get_note_by_id(&self, id: u64) -> Result<Note, Error> {
        match self.notes_storage.get(&id) {
            Some(note) => Ok(note.clone()),
            None => Err(Error::NotFound {
                msg: format!("A note with id={} not found", id),
            }),
        }
    }

    fn add_note(&self, note_payload: NotePayload) -> Result<Note, Error> {
        let mut id_counter = self.id_counter.lock().expect("Mutex lock failed");
        let id = *id_counter;
        *id_counter += 1;

        let note = Note {
            id,
            title: note_payload.title,
            content: note_payload.content,
            created_at: time(),
            updated_at: None,
        };

        self.notes_storage.insert(note.id, note.clone());
        Ok(note)
    }

    fn update_note(&self, id: u64, payload: NotePayload) -> Result<Note, Error> {
        if let Some(mut note) = self.notes_storage.get(&id).cloned() {
            note.content = payload.content;
            note.title = payload.title;
            note.updated_at = Some(time());
            self.notes_storage.insert(note.id, note.clone());
            Ok(note)
        } else {
            Err(Error::NotFound {
                msg: format!("Couldn't update a note with id={}. Note not found", id),
            })
        }
    }

    fn delete_note(&self, id: u64) -> Result<Note, Error> {
        if let Some(note) = self.notes_storage.remove(&id) {
            Ok(note)
        } else {
            Err(Error::NotFound {
                msg: format!("Couldn't delete a note with id={}. Note not found", id),
            })
        }
    }
}

ic_cdk::export_candid!();
