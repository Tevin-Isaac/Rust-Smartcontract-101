import { note_backend } from "../../declarations/note_backend";

// Function to add a new note
async function addNote() {
  const title = document.getElementById("title").value.toString();
  const content = document.getElementById("content").value.toString();

  try {
    // Interact with the note_backend canister, calling the add_note method
    const addedNote = await note_backend.add_note({ title, content });

    // Optionally, you can update the UI to display the added note or perform other actions
    console.log('Note added:', addedNote);

    // Fetch all notes after adding a new one
    await getAllNotes();
  } catch (error) {
    // Handle errors
    console.error('Error adding note:', error);
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
    // Handle errors
    console.error('Error getting all notes:', error);
  }
}

// Event listener for form submission
document.querySelector("#addNoteForm").addEventListener("submit", async (e) => {
  e.preventDefault();
  await addNote();
});

// Optional: Call getAllNotes on page load
window.onload = function () {
  getAllNotes();
};
