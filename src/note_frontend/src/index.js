import { note_backend } from "../../declarations/note_backend";

// Function to add a new note
async function addNote() {
  const titleElement = document.getElementById("title");
  const contentElement = document.getElementById("content");

  // Input validation
  const title = titleElement.value.trim();
  const content = contentElement.value.trim();

  if (!title || !content) {
    console.error('Title and content cannot be empty');
    return;
  }

  try {
    // Interact with the note_backend canister, calling the add_note method
    const addedNote = await note_backend.add_note({ title, content });

    // Optionally, you can update the UI to display the added note or perform other actions
    console.log('Note added:', addedNote);

    // Fetch all notes after adding a new one
    await getAllNotes();
  } catch (error) {
    // Handle errors with more details
    console.error('Error adding note:', error.message || error);
    // Optionally display an error message to the user
  }
}

// Function to fetch all notes and update the UI
async function getAllNotes() {
  try {
    // Interact with the note_backend canister, calling the get_all_notes method
    const notes = await note_backend.get_all_notes();

    // Optionally, you can update the UI to display the retrieved notes or perform other actions
    console.log('All Notes:', notes);
  } catch (error) {
    // Handle errors with more details
    console.error('Error getting all notes:', error.message || error);
    // Optionally display an error message to the user
  }
}

// Event listener for form submission
document.querySelector("#addNoteForm").addEventListener("submit", async (e) => {
  e.preventDefault();
  try {
    await addNote();
  } catch (error) {
    // Handle errors with more details
    console.error('Error submitting form:', error.message || error);
    // Optionally display an error message to the user
  }
});

// Optional: Call getAllNotes on window load
window.addEventListener('load', function () {
  getAllNotes();
});
