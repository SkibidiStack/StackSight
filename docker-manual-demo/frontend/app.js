const clock = document.getElementById('clock');
const statusEl = document.getElementById('status');
const noteInput = document.getElementById('noteInput');
const addNoteBtn = document.getElementById('addNoteBtn');
const notesList = document.getElementById('notesList');
const simulateBtn = document.getElementById('simulateBtn');
const clearBtn = document.getElementById('clearBtn');

function updateClock() {
  clock.textContent = new Date().toLocaleTimeString();
}

function setStatus(text) {
  statusEl.textContent = text;
}

function addNote(text) {
  const li = document.createElement('li');
  li.textContent = text;
  notesList.appendChild(li);
}

addNoteBtn.addEventListener('click', () => {
  const text = noteInput.value.trim();
  if (!text) {
    setStatus('Please type a note first');
    return;
  }

  addNote(text);
  noteInput.value = '';
  setStatus(`Added note (${notesList.children.length})`);
});

simulateBtn.addEventListener('click', () => {
  setStatus('Building...');
  window.setTimeout(() => {
    setStatus('Build step completed successfully');
  }, 600);
});

clearBtn.addEventListener('click', () => {
  notesList.innerHTML = '';
  setStatus('Notes cleared');
});

updateClock();
window.setInterval(updateClock, 1000);
