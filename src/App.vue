<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const greetMsg = ref("");
const name = ref("");
const chatHistory = ref([]);
const showHistory = ref(false);

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsg.value = await invoke("greet", { name: name.value });
}

async function loadChatHistory() {
  try {
    chatHistory.value = await invoke("load_chat_history");
    showHistory.value = true;
  } catch (err) {
    console.error("Failed to load chat history:", err);
  }
}

function closeChatHistory() {
  showHistory.value = false;
}
</script>

<template>
  <main class="container">
    <h1>Welcome to Tauri + Vue</h1>

    <div class="row">
      <a href="https://vite.dev" target="_blank">
        <img src="/vite.svg" class="logo vite" alt="Vite logo" />
      </a>
      <a href="https://tauri.app" target="_blank">
        <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />
      </a>
      <a href="https://vuejs.org/" target="_blank">
        <img src="./assets/vue.svg" class="logo vue" alt="Vue logo" />
      </a>
    </div>
    <p>Click on the Tauri, Vite, and Vue logos to learn more.</p>


    <form class="row" @submit.prevent="greet">
      <input id="greet-input" v-model="name" placeholder="Enter a name..." />
      <button type="submit">Greet</button>
    </form>
    <p>{{ greetMsg }}</p>

    <div class="chat-history-section">
      <button @click="loadChatHistory" class="history-btn">
        📚 View Chat History
      </button>
      
      <!-- Chat History Modal -->
      <div v-if="showHistory" class="modal-overlay" @click="closeChatHistory">
        <div class="modal-content" @click.stop>
          <div class="modal-header">
            <h2>📚 Chat History</h2>
            <button @click="closeChatHistory" class="close-modal-btn">×</button>
          </div>
          <div class="modal-body">
            <div v-if="chatHistory.length === 0" class="no-history">
              No chat history found. Use Ctrl+Space to capture and process text!
            </div>
            <div v-else class="history-list">
              <div v-for="(entry, index) in chatHistory" :key="index" class="history-entry">
                <div class="entry-timestamp">{{ entry.timestamp }}</div>
                <div class="entry-content">
                  <div class="original-text">
                    <strong>Original:</strong> {{ entry.original_text }}
                  </div>
                  <div class="ai-option">
                    <strong>Action:</strong> {{ entry.ai_option }}
                  </div>
                  <div class="processed-text">
                    <strong>Result:</strong> {{ entry.processed_text }}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </main>
</template>

<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}

</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}
button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

.chat-history-section {
  margin-top: 30px;
}

.history-btn {
  background-color: #2196F3;
  color: white;
  border: none;
  padding: 12px 24px;
  font-size: 16px;
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.3s;
}

.history-btn:hover {
  background-color: #1976D2;
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal-content {
  background: white;
  border-radius: 12px;
  width: 90%;
  max-width: 800px;
  max-height: 90%;
  overflow: hidden;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.3);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #eee;
}

.modal-header h2 {
  margin: 0;
  color: #2196F3;
}

.close-modal-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #666;
  padding: 0;
  width: 30px;
  height: 30px;
}

.modal-body {
  padding: 20px;
  max-height: 500px;
  overflow-y: auto;
}

.no-history {
  text-align: center;
  color: #666;
  font-style: italic;
  padding: 40px 20px;
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.history-entry {
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 16px;
  background-color: #f9f9f9;
}

.entry-timestamp {
  font-size: 12px;
  color: #666;
  margin-bottom: 8px;
}

.entry-content {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.original-text {
  color: #333;
}

.ai-option {
  color: #2196F3;
  font-size: 14px;
}

.processed-text {
  color: #4CAF50;
}

@media (prefers-color-scheme: dark) {
  .modal-content {
    background: #2f2f2f;
    color: #f6f6f6;
  }
  
  .modal-header {
    border-bottom-color: #555;
  }
  
  .history-entry {
    background-color: #3a3a3a;
    border-color: #555;
  }
  
  .close-modal-btn {
    color: #ccc;
  }
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }
  button:active {
    background-color: #0f0f0f69;
  }
}

</style>