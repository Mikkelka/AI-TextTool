import { createApp } from "vue";
import ChatWindow from "./components/ChatWindow.vue";

// Get parameters from URL
const urlParams = new URLSearchParams(window.location.search);
const operation = urlParams.get('operation') || 'Chat';
const initialText = urlParams.get('text') || '';
const title = urlParams.get('title') || 'AI Chat';
const instruction = urlParams.get('instruction') || '';
const conversationId = urlParams.get('conversationId') || '';

// Create Vue app instance with props
const app = createApp(ChatWindow, {
  operation: operation,
  initialText: initialText,
  title: title,
  instruction: instruction,
  conversationId: conversationId
});

// Mount the app
app.mount("#app");