const output = document.getElementById('output');

function show(data) {
  output.textContent = JSON.stringify(data, null, 2);
}

async function callApi(path, options = {}) {
  const res = await fetch(path, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
  });
  return res.json();
}

document.getElementById('healthBtn').addEventListener('click', async () => {
  try {
    const data = await callApi('/api/health');
    show(data);
  } catch (err) {
    show({ error: String(err) });
  }
});

document.getElementById('messageBtn').addEventListener('click', async () => {
  try {
    const data = await callApi('/api/message');
    show(data);
  } catch (err) {
    show({ error: String(err) });
  }
});

document.getElementById('echoBtn').addEventListener('click', async () => {
  const text = document.getElementById('echoInput').value;
  try {
    const data = await callApi('/api/echo', {
      method: 'POST',
      body: JSON.stringify({ text }),
    });
    show(data);
  } catch (err) {
    show({ error: String(err) });
  }
});
