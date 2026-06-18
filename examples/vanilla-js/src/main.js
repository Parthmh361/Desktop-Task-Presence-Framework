import { DTPFClient } from '@dtpf/sdk-core';
import './style.css';

const statusEl = document.getElementById('status');
const createBtn = document.getElementById('create-btn');
const messageEl = document.getElementById('message');

const client = new DTPFClient({
  appId: 'com.acme.tasks.vanilla',
  appName: 'Vanilla JS Demo',
});

function setStatus(text, className = '') {
  statusEl.textContent = text;
  statusEl.className = className;
}

function showMessage(text) {
  messageEl.textContent = text;
  messageEl.hidden = false;
  setTimeout(() => {
    messageEl.hidden = true;
  }, 3000);
}

createBtn.addEventListener('click', async () => {
  createBtn.disabled = true;
  try {
    await client.createStickyTask({
      title: 'Fix login bug',
      body: 'Investigate OAuth redirect loop on Safari',
      priority: 2,
      color: '#FFE066',
    });
    showMessage('Sticky note created on your desktop!');
  } catch (err) {
    showMessage(err instanceof Error ? err.message : String(err));
  } finally {
    createBtn.disabled = false;
  }
});

client
  .connect()
  .then(async () => {
    const agentStatus = await client.getAgentStatus();
    setStatus(
      `Connected · Agent v${agentStatus.version} · ${agentStatus.taskCount} stickies`,
      'connected',
    );
    createBtn.disabled = false;
  })
  .catch((err) => {
    setStatus(
      `DTPF Agent not connected. ${err instanceof Error ? err.message : String(err)}`,
      'error',
    );
  });
