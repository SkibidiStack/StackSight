const express = require('express');
const cors = require('cors');

const app = express();
const port = 5000;

app.use(cors());
app.use(express.json());

app.get('/api/health', (req, res) => {
  res.json({ status: 'ok', service: 'demo-backend' });
});

app.get('/api/message', (req, res) => {
  res.json({ message: 'Hello from the backend container!' });
});

app.post('/api/echo', (req, res) => {
  const text = req.body?.text ?? '';
  res.json({ echo: text, length: text.length });
});

app.listen(port, '0.0.0.0', () => {
  console.log(`Demo backend listening on port ${port}`);
});
