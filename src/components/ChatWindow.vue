<template>
  <div class="chat-window" @keydown="handleGlobalKeydown">
    <!-- Header -->
    <div class="chat-header">
      <div class="header-left">
        <h1 class="chat-title">{{ windowTitle }}</h1>
        <div class="operation-info" v-if="operation">
          <span class="operation-badge">{{ operation }}</span>
        </div>
      </div>
      
      <div class="header-controls">
        <!-- Model Selector -->
        <div class="model-selector">
          <label for="model-select">Model:</label>
          <select 
            id="model-select"
            v-model="selectedModel" 
            class="model-dropdown"
            :disabled="isProcessing"
          >
            <option v-for="model in availableModels" :key="model" :value="model">
              {{ formatModelName(model) }}
            </option>
          </select>
        </div>
        
        <!-- Thinking Toggle -->
        <div class="thinking-toggle" v-if="supportsThinking">
          <label class="toggle-label">
            <input 
              type="checkbox" 
              v-model="enableThinking"
              :disabled="isProcessing"
            />
            <span class="toggle-slider"></span>
            Thinking Mode
          </label>
        </div>
        
        <!-- Action Buttons -->
        <div class="action-buttons">
          <button 
            @click="saveConversation" 
            class="action-btn save-btn"
            :disabled="messages.length === 0"
            title="Save conversation to history"
          >
            💾
          </button>
          <button 
            @click="clearConversation" 
            class="action-btn clear-btn"
            :disabled="messages.length === 0"
            title="Clear conversation (Ctrl+L)"
          >
            🗑️
          </button>
          <button 
            @click="zoomIn" 
            class="action-btn zoom-btn"
            title="Zoom in (Ctrl+Plus)"
          >
            🔍+
          </button>
          <button 
            @click="zoomOut" 
            class="action-btn zoom-btn"
            title="Zoom out (Ctrl+Minus)"
          >
            🔍-
          </button>
          <button 
            @click="closeWindow" 
            class="action-btn close-btn"
            title="Close window (Escape)"
          >
            ✕
          </button>
        </div>
      </div>
    </div>

    <!-- Chat Messages Area -->
    <div class="chat-messages" ref="messagesContainer">
      <!-- Welcome Message -->
      <div v-if="messages.length === 0" class="welcome-message">
        <div class="welcome-content">
          <h2>💬 AI Chat Assistant</h2>
          <p v-if="initialText">Ready to process your text and answer follow-up questions.</p>
          <p v-else>Start a conversation by typing your message below.</p>
          <div class="initial-text-preview" v-if="initialText">
            <strong>Selected text:</strong>
            <div class="initial-text">{{ initialTextPreview }}</div>
          </div>
        </div>
      </div>

      <!-- Message Bubbles -->
      <div v-for="(message, index) in messages" :key="index" class="message-wrapper">
        <div 
          :class="[
            'message-bubble',
            `message-${message.role}`,
            { 'message-processing': message.isProcessing }
          ]"
        >
          <!-- Message Header -->
          <div class="message-header">
            <span class="message-role">
              {{ message.role === 'user' ? '👤 You' : '🤖 AI Assistant' }}
            </span>
            <span class="message-time">
              {{ formatTime(message.timestamp) }}
            </span>
            <div class="message-actions">
              <button 
                @click="copyMessage(message.content)" 
                class="copy-btn"
                title="Copy message"
              >
                📋
              </button>
              <button 
                v-if="message.role === 'assistant' && !message.isProcessing"
                @click="regenerateResponse(index)"
                class="regenerate-btn"
                title="Regenerate response"
              >
                🔄
              </button>
            </div>
          </div>

          <!-- Thinking Process (only for assistant messages with thoughts) -->
          <div 
            v-if="message.role === 'assistant' && message.thoughts && !message.isProcessing" 
            class="thoughts-section"
          >
            <details class="thoughts-details">
              <summary class="thoughts-header">
                💭 AI's Thinking Process
              </summary>
              <div class="thoughts-content">
                <div 
                  class="markdown-content thoughts-markdown"
                  v-html="renderMarkdown(message.thoughts)"
                ></div>
              </div>
            </details>
          </div>

          <!-- Message Content -->
          <div class="message-content">
            <div 
              v-if="message.role === 'assistant'" 
              class="markdown-content"
              v-html="renderMarkdown(message.content)"
            ></div>
            <div v-else class="user-content">
              {{ message.content }}
            </div>
          </div>

          <!-- Processing Indicator -->
          <div v-if="message.isProcessing" class="processing-indicator">
            <div class="thinking-dots">
              <span></span>
              <span></span>
              <span></span>
            </div>
            <span class="processing-text">AI is thinking...</span>
          </div>
        </div>
      </div>

      <!-- Typing Indicator -->
      <div v-if="isProcessing && !messages.some(m => m.isProcessing)" class="typing-indicator">
        <div class="typing-bubble">
          <div class="thinking-dots">
            <span></span>
            <span></span>
            <span></span>
          </div>
          <span class="typing-text">AI is typing...</span>
        </div>
      </div>
    </div>

    <!-- Input Area -->
    <div class="chat-input-area">
      <div class="input-container">
        <textarea
          ref="messageInput"
          v-model="currentMessage"
          @keydown="handleInputKeydown"
          placeholder="Type your message here... (Press Enter to send, Ctrl+Enter for new line)"
          class="message-input"
          :disabled="isProcessing"
          rows="1"
        ></textarea>
        <button 
          @click="sendMessage"
          :disabled="!canSend"
          class="send-button"
          title="Send message (Enter)"
        >
          <span v-if="!isProcessing">📤</span>
          <span v-else class="spinner">⏳</span>
        </button>
      </div>
      
      <!-- Input Status -->
      <div class="input-status">
        <div class="character-count">
          {{ currentMessage.length }} characters
        </div>
        <div class="input-hints">
          <span class="hint">Enter to send</span>
          <span class="hint">Ctrl+Enter for new line</span>
          <span class="hint">Escape to close</span>
        </div>
      </div>
    </div>

    <!-- Error Display -->
    <div v-if="error" class="error-message">
      <div class="error-content">
        <span class="error-icon">⚠️</span>
        <span class="error-text">{{ error }}</span>
        <button @click="clearError" class="error-close">✕</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { renderMarkdown, setupMarkdownCopyFunction, cleanupMarkdownCopyFunction } from '../utils/markdown'

// Props
interface Props {
  operation?: string
  initialText?: string
  title?: string
  instruction?: string
  conversationId?: string
}

const props = withDefaults(defineProps<Props>(), {
  operation: '',
  initialText: '',
  title: 'AI Chat',
  instruction: '',
  conversationId: ''
})

// Message Interface
interface ChatMessage {
  role: 'user' | 'assistant'
  content: string
  timestamp: string
  isProcessing?: boolean
  thoughts?: string
}

// Reactive State
const messages = ref<ChatMessage[]>([])
const currentMessage = ref('')
const isProcessing = ref(false)
const error = ref<string | null>(null)
const selectedModel = ref('gemini-2.5-flash')
const enableThinking = ref(false)
const availableModels = ref<string[]>([])
const messagesContainer = ref<HTMLElement>()
const messageInput = ref<HTMLTextAreaElement>()
const zoomLevel = ref(100)

// Computed Properties
const windowTitle = computed(() => {
  if (props.title && props.title !== 'AI Chat') return props.title
  return props.operation ? `AI Chat - ${props.operation}` : 'AI Chat'
})

const initialTextPreview = computed(() => {
  if (!props.initialText) return ''
  const maxLength = 150
  return props.initialText.length > maxLength 
    ? props.initialText.substring(0, maxLength) + '...'
    : props.initialText
})

const canSend = computed(() => {
  return currentMessage.value.trim().length > 0 && !isProcessing.value
})

const supportsThinking = computed(() => {
  return ['gemini-2.5-flash', 'gemini-2.5-flash-lite'].includes(selectedModel.value)
})

// Methods
const loadAvailableModels = async () => {
  try {
    const models = await invoke('get_ai_models') as string[]
    availableModels.value = models
  } catch (err) {
    console.error('Failed to load models:', err)
    availableModels.value = ['gemini-2.5-flash', 'gemini-2.5-flash-lite']
  }
}

const formatModelName = (model: string): string => {
  return model
    .replace('gemini-', 'Gemini ')
    .replace('-', ' ')
    .split(' ')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ')
}

const formatTime = (timestamp: string): string => {
  try {
    const date = new Date(timestamp)
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  } catch {
    return new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  }
}

const sendMessage = async () => {
  if (!canSend.value) return

  const userMessage = currentMessage.value.trim()
  currentMessage.value = ''
  error.value = null

  // Add user message
  const userMsg: ChatMessage = {
    role: 'user',
    content: userMessage,
    timestamp: new Date().toISOString()
  }
  messages.value.push(userMsg)

  // Add processing AI message
  const aiMsg: ChatMessage = {
    role: 'assistant',
    content: '',
    timestamp: new Date().toISOString(),
    isProcessing: true
  }
  messages.value.push(aiMsg)

  isProcessing.value = true
  await scrollToBottom()

  try {
    // Prepare message history (exclude processing message)
    const chatHistory = messages.value
      .filter(m => !m.isProcessing)
      .map(m => ({
        role: m.role,
        content: m.content,
        timestamp: m.timestamp
      }))

    // Include initial text in first user message if this is the first interaction
    let messageToSend = userMessage
    if (messages.value.length === 2 && props.initialText) {
      messageToSend = `${props.initialText}\n\n${userMessage}`
    }

    // Call backend AI service with custom instruction if available
    const response = await invoke('chat_with_ai', {
      message: messageToSend,
      history: chatHistory.slice(0, -1), // Exclude the current user message since it's included separately
      customInstruction: props.instruction || null,
      enableThinking: enableThinking.value
    }) as { answer: string, thoughts?: string }

    // Update AI message with response
    const aiIndex = messages.value.findIndex(m => m.isProcessing)
    if (aiIndex !== -1) {
      messages.value[aiIndex] = {
        role: 'assistant',
        content: response.answer,
        timestamp: new Date().toISOString(),
        isProcessing: false,
        thoughts: response.thoughts
      }
    }

  } catch (err) {
    console.error('Failed to get AI response:', err)
    error.value = err instanceof Error ? err.message : 'Failed to get AI response'
    
    // Remove processing message on error
    const processingIndex = messages.value.findIndex(m => m.isProcessing)
    if (processingIndex !== -1) {
      messages.value.splice(processingIndex, 1)
    }
  } finally {
    isProcessing.value = false
    await scrollToBottom()
    focusInput()
  }
}

const regenerateResponse = async (messageIndex: number) => {
  if (messageIndex <= 0 || isProcessing.value) return

  const userMessage = messages.value[messageIndex - 1]
  if (userMessage.role !== 'user') return

  // Remove the AI response and any subsequent messages
  messages.value = messages.value.slice(0, messageIndex)
  
  // Add processing indicator
  const aiMsg: ChatMessage = {
    role: 'assistant',
    content: '',
    timestamp: new Date().toISOString(),
    isProcessing: true
  }
  messages.value.push(aiMsg)

  isProcessing.value = true
  await scrollToBottom()

  try {
    const chatHistory = messages.value
      .filter(m => !m.isProcessing)
      .map(m => ({
        role: m.role,
        content: m.content,
        timestamp: m.timestamp
      }))

    const response = await invoke('chat_with_ai', {
      message: userMessage.content,
      history: chatHistory.slice(0, -1),
      customInstruction: props.instruction || null,
      enableThinking: enableThinking.value
    }) as { answer: string, thoughts?: string }

    // Update AI message
    const aiIndex = messages.value.findIndex(m => m.isProcessing)
    if (aiIndex !== -1) {
      messages.value[aiIndex] = {
        role: 'assistant',
        content: response.answer,
        timestamp: new Date().toISOString(),
        isProcessing: false,
        thoughts: response.thoughts
      }
    }

  } catch (err) {
    console.error('Failed to regenerate response:', err)
    error.value = err instanceof Error ? err.message : 'Failed to regenerate response'
    
    // Remove processing message
    const processingIndex = messages.value.findIndex(m => m.isProcessing)
    if (processingIndex !== -1) {
      messages.value.splice(processingIndex, 1)
    }
  } finally {
    isProcessing.value = false
    await scrollToBottom()
  }
}

const copyMessage = async (content: string) => {
  try {
    await navigator.clipboard.writeText(content)
    // Could add a toast notification here
  } catch (err) {
    console.error('Failed to copy message:', err)
  }
}

const saveConversation = async () => {
  if (messages.value.length === 0) return

  try {
    // Generate a smart default title from first user message
    const firstUserMessage = messages.value.find(m => m.role === 'user')
    const defaultTitle = firstUserMessage ? 
      (firstUserMessage.content.length > 50 ? 
        firstUserMessage.content.substring(0, 50) + '...' : 
        firstUserMessage.content
      ) : 'Untitled Conversation'

    // Prompt user for conversation title
    const title = prompt(`Save conversation as:`, defaultTitle)
    if (!title) return // User cancelled

    // Convert messages to ConversationMessage format
    const conversationMessages = messages.value
      .filter(msg => !msg.isProcessing) // Exclude any processing messages
      .map(msg => ({
        role: msg.role,
        content: msg.content,
        timestamp: msg.timestamp,
        thoughts: msg.thoughts
      }))

    // Save the full conversation
    const conversationId = await invoke('save_conversation', {
      title: title.trim(),
      operation: props.operation || 'Chat',
      messages: conversationMessages,
      thinkingModeEnabled: enableThinking.value
    }) as string

    console.log('Conversation saved successfully with ID:', conversationId)
    
    // Show success feedback (simple alert for now)
    alert('✅ Conversation saved successfully!')
    
  } catch (err) {
    console.error('Failed to save conversation:', err)
    error.value = 'Failed to save conversation: ' + (err instanceof Error ? err.message : String(err))
    
    // Show error feedback
    alert('❌ Failed to save conversation: ' + error.value)
  }
}

const clearConversation = () => {
  if (messages.value.length === 0) return
  
  if (confirm('Are you sure you want to clear this conversation?')) {
    messages.value = []
    error.value = null
    focusInput()
  }
}

const clearError = () => {
  error.value = null
}

const scrollToBottom = async () => {
  await nextTick()
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
  }
}

const focusInput = async () => {
  await nextTick()
  messageInput.value?.focus()
}

const zoomIn = () => {
  if (zoomLevel.value < 200) {
    zoomLevel.value += 10
    applyZoom()
  }
}

const zoomOut = () => {
  if (zoomLevel.value > 50) {
    zoomLevel.value -= 10
    applyZoom()
  }
}

const applyZoom = () => {
  document.documentElement.style.fontSize = `${zoomLevel.value}%`
}

const closeWindow = () => {
  getCurrentWindow().close()
}

const handleGlobalKeydown = (event: KeyboardEvent) => {
  if (event.ctrlKey || event.metaKey) {
    switch (event.key) {
      case 'l':
      case 'L':
        event.preventDefault()
        clearConversation()
        break
      case '=':
      case '+':
        event.preventDefault()
        zoomIn()
        break
      case '-':
        event.preventDefault()
        zoomOut()
        break
    }
  } else if (event.key === 'Escape') {
    closeWindow()
  }
}

const handleInputKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Enter' && !event.shiftKey && !event.ctrlKey) {
    event.preventDefault()
    sendMessage()
  } else if (event.key === 'Enter' && event.ctrlKey) {
    // Allow new line
    return
  }
  
  // Auto-resize textarea
  nextTick(() => {
    if (messageInput.value) {
      messageInput.value.style.height = 'auto'
      messageInput.value.style.height = Math.min(messageInput.value.scrollHeight, 120) + 'px'
    }
  })
}

// Use imported markdown renderer from utils

// Load existing conversation
const loadConversation = async () => {
  if (!props.conversationId) return
  
  try {
    console.log('Loading conversation:', props.conversationId)
    
    // Load the conversation from backend
    const conversation = await invoke('load_conversation_messages', {
      conversationId: props.conversationId
    }) as {
      id: string
      title: string
      operation: string
      messages: Array<{
        role: string
        content: string
        timestamp: string
        thoughts?: string
      }>
      created_at: string
      updated_at: string
      thinking_mode_enabled?: boolean
    }
    
    // Convert and load messages
    messages.value = conversation.messages.map(msg => ({
      role: msg.role as 'user' | 'assistant',
      content: msg.content,
      timestamp: msg.timestamp,
      isProcessing: false,
      thoughts: msg.thoughts
    }))
    
    // Restore thinking mode setting from saved conversation
    if (conversation.thinking_mode_enabled !== undefined) {
      enableThinking.value = conversation.thinking_mode_enabled
      console.log(`Restored thinking mode: ${conversation.thinking_mode_enabled}`)
    }
    
    console.log(`Loaded conversation "${conversation.title}" with ${conversation.messages.length} messages`)
    
    // Scroll to bottom after messages are loaded
    await nextTick()
    scrollToBottom()
    
  } catch (err) {
    console.error('Failed to load conversation:', err)
    error.value = 'Failed to load conversation: ' + (err instanceof Error ? err.message : String(err))
  }
}

// Auto-scroll on new messages
watch(() => messages.value.length, () => {
  scrollToBottom()
})

// Focus input when not processing
watch(isProcessing, (newVal) => {
  if (!newVal) {
    focusInput()
  }
})

// Lifecycle
onMounted(async () => {
  console.log('ChatWindow mounted with props:', {
    operation: props.operation,
    initialText: props.initialText ? `${props.initialText.length} chars` : 'none',
    title: props.title,
    conversationId: props.conversationId
  })
  
  await loadAvailableModels()
  focusInput()
  
  // Parse URL parameters if not provided as props (fallback)
  const urlParams = new URLSearchParams(window.location.search)
  const operation = props.operation || urlParams.get('operation') || ''
  const initialText = props.initialText || urlParams.get('text') || ''
  
  console.log('Final values after URL parsing:', {
    operation,
    initialText: initialText ? `${initialText.length} chars` : 'none'
  })
  
  // Load existing conversation if conversationId is provided
  if (props.conversationId) {
    console.log('Loading existing conversation:', props.conversationId)
    await loadConversation()
  } else {
    // Send initial message if there's initial text and operation (only for new chats)
    if (initialText && operation) {
      console.log(`Auto-sending initial text for operation: ${operation}`)
      // Send the actual selected text as the first message
      currentMessage.value = initialText
      await sendMessage()
    } else {
      console.log('No initial text or operation - waiting for user input')
    }
  }
  
  // Setup global markdown copy function
  setupMarkdownCopyFunction()
})

onUnmounted(() => {
  // Cleanup markdown copy function
  cleanupMarkdownCopyFunction()
})
</script>

<style scoped>
.chat-window {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

/* Header */
.chat-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  background: rgba(255, 255, 255, 0.95);
  border-bottom: 1px solid rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(10px);
  z-index: 10;
}

.header-left {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.chat-title {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
  color: #333;
}

.operation-info {
  display: flex;
  gap: 8px;
}

.operation-badge {
  background: #e3f2fd;
  color: #1976d2;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 500;
  text-transform: capitalize;
}

.header-controls {
  display: flex;
  align-items: center;
  gap: 16px;
}

.model-selector {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}

.model-selector label {
  color: #666;
  font-weight: 500;
}

.model-dropdown {
  padding: 4px 8px;
  border: 1px solid #ddd;
  border-radius: 6px;
  background: white;
  font-size: 13px;
  color: #333;
  cursor: pointer;
}

.model-dropdown:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.thinking-toggle {
  display: flex;
  align-items: center;
}

.toggle-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: #666;
  cursor: pointer;
  user-select: none;
}

.toggle-label input {
  display: none;
}

.toggle-slider {
  position: relative;
  width: 36px;
  height: 20px;
  background: #ddd;
  border-radius: 10px;
  transition: background 0.3s;
}

.toggle-slider::before {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  background: white;
  border-radius: 50%;
  transition: transform 0.3s;
}

.toggle-label input:checked + .toggle-slider {
  background: #2196f3;
}

.toggle-label input:checked + .toggle-slider::before {
  transform: translateX(16px);
}

.action-buttons {
  display: flex;
  gap: 6px;
}

.action-btn {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  transition: all 0.2s;
  background: rgba(255, 255, 255, 0.8);
}

.action-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 1);
  transform: scale(1.05);
}

.action-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.save-btn { color: #4caf50; }
.clear-btn { color: #f44336; }
.zoom-btn { color: #2196f3; }
.close-btn { color: #666; }

/* Messages Area */
.chat-messages {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.welcome-message {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 200px;
  text-align: center;
  color: #666;
}

.welcome-content h2 {
  color: #333;
  margin-bottom: 16px;
}

.initial-text-preview {
  margin-top: 16px;
  padding: 16px;
  background: rgba(255, 255, 255, 0.8);
  border-radius: 8px;
  border-left: 4px solid #2196f3;
  text-align: left;
  max-width: 400px;
}

.initial-text {
  margin-top: 8px;
  padding: 8px;
  background: #f5f5f5;
  border-radius: 4px;
  font-family: monospace;
  font-size: 13px;
  white-space: pre-wrap;
  word-break: break-word;
}

.message-wrapper {
  display: flex;
  flex-direction: column;
}

.message-bubble {
  max-width: 80%;
  padding: 16px;
  border-radius: 18px;
  position: relative;
  word-wrap: break-word;
}

.message-user {
  align-self: flex-end;
  background: linear-gradient(135deg, #e3f2fd 0%, #bbdefb 100%);
  border-bottom-right-radius: 6px;
}

.message-assistant {
  align-self: flex-start;
  background: rgba(255, 255, 255, 0.95);
  border-bottom-left-radius: 6px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.message-processing {
  opacity: 0.8;
}

.message-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  font-size: 12px;
}

.message-role {
  font-weight: 600;
  color: #666;
}

.message-time {
  color: #999;
}

.message-actions {
  display: flex;
  gap: 4px;
}

.copy-btn,
.regenerate-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
  font-size: 12px;
  opacity: 0.6;
  transition: opacity 0.2s;
}

.copy-btn:hover,
.regenerate-btn:hover {
  opacity: 1;
  background: rgba(0, 0, 0, 0.05);
}

.message-content {
  line-height: 1.5;
}

.user-content {
  color: #333;
}

.markdown-content {
  color: #333;
}

.markdown-content :deep(h1),
.markdown-content :deep(h2),
.markdown-content :deep(h3) {
  margin: 16px 0 8px 0;
  color: #333;
}

.markdown-content :deep(p) {
  margin: 8px 0;
}

.markdown-content :deep(pre) {
  position: relative;
  background: #f8f9fa;
  border: 1px solid #e9ecef;
  border-radius: 6px;
  padding: 12px;
  margin: 12px 0;
  overflow-x: auto;
}

.markdown-content :deep(.copy-code-btn) {
  position: absolute;
  top: 8px;
  right: 8px;
  background: rgba(255, 255, 255, 0.8);
  border: 1px solid #ddd;
  border-radius: 4px;
  padding: 4px 8px;
  font-size: 12px;
  cursor: pointer;
}

.markdown-content :deep(code) {
  font-family: 'Monaco', 'Consolas', monospace;
  font-size: 13px;
}

.markdown-content :deep(.inline-code) {
  background: #f1f3f4;
  padding: 2px 4px;
  border-radius: 3px;
  font-size: 13px;
}

.markdown-content :deep(ul) {
  margin: 8px 0;
  padding-left: 24px;
}

.markdown-content :deep(li) {
  margin: 4px 0;
}

.markdown-content :deep(a) {
  color: #1976d2;
  text-decoration: none;
}

.markdown-content :deep(a:hover) {
  text-decoration: underline;
}

/* Markdown Headings */
.markdown-content :deep(.markdown-heading) {
  margin: 16px 0 8px 0;
  font-weight: 600;
  line-height: 1.2;
}

.markdown-content :deep(.markdown-h1) {
  font-size: 24px;
  color: #333;
  border-bottom: 2px solid #e0e0e0;
  padding-bottom: 8px;
}

.markdown-content :deep(.markdown-h2) {
  font-size: 20px;
  color: #333;
}

.markdown-content :deep(.markdown-h3) {
  font-size: 18px;
  color: #444;
}

.markdown-content :deep(.markdown-h4) {
  font-size: 16px;
  color: #555;
}

.markdown-content :deep(.markdown-h5) {
  font-size: 14px;
  color: #666;
}

.markdown-content :deep(.markdown-h6) {
  font-size: 13px;
  color: #777;
}

/* Markdown Tables */
.markdown-content :deep(.table-wrapper) {
  overflow-x: auto;
  margin: 12px 0;
}

.markdown-content :deep(.markdown-table) {
  width: 100%;
  border-collapse: collapse;
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  overflow: hidden;
}

.markdown-content :deep(.markdown-table th) {
  background: #f5f5f5;
  padding: 8px 12px;
  text-align: left;
  font-weight: 600;
  border-bottom: 1px solid #e0e0e0;
  border-right: 1px solid #e0e0e0;
}

.markdown-content :deep(.markdown-table th:last-child) {
  border-right: none;
}

.markdown-content :deep(.markdown-table td) {
  padding: 8px 12px;
  border-bottom: 1px solid #f0f0f0;
  border-right: 1px solid #f0f0f0;
}

.markdown-content :deep(.markdown-table td:last-child) {
  border-right: none;
}

.markdown-content :deep(.markdown-table tr:last-child td) {
  border-bottom: none;
}

.markdown-content :deep(.markdown-table tr:nth-child(even)) {
  background: #fafafa;
}

/* Markdown Blockquotes */
.markdown-content :deep(.markdown-blockquote) {
  margin: 12px 0;
  padding: 12px 16px;
  border-left: 4px solid #2196f3;
  background: rgba(33, 150, 243, 0.05);
  border-radius: 0 6px 6px 0;
  color: #555;
  font-style: italic;
}

.markdown-content :deep(.markdown-blockquote p) {
  margin: 0;
}

/* Markdown Lists */
.markdown-content :deep(.markdown-list) {
  margin: 8px 0;
  padding-left: 24px;
}

.markdown-content :deep(.markdown-list li) {
  margin: 4px 0;
  line-height: 1.5;
}

/* Horizontal Rules */
.markdown-content :deep(hr) {
  margin: 16px 0;
  border: none;
  border-top: 1px solid #e0e0e0;
}

.processing-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
  color: #666;
  font-size: 13px;
}

.typing-indicator {
  align-self: flex-start;
  margin: 8px 0;
}

.typing-bubble {
  background: rgba(255, 255, 255, 0.9);
  padding: 12px 16px;
  border-radius: 18px;
  border-bottom-left-radius: 6px;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: #666;
}

.thinking-dots {
  display: flex;
  gap: 2px;
}

.thinking-dots span {
  width: 6px;
  height: 6px;
  background: #999;
  border-radius: 50%;
  animation: thinking 1.4s infinite ease-in-out;
}

.thinking-dots span:nth-child(1) {
  animation-delay: -0.32s;
}

.thinking-dots span:nth-child(2) {
  animation-delay: -0.16s;
}

@keyframes thinking {
  0%, 80%, 100% {
    transform: scale(0.6);
    opacity: 0.4;
  }
  40% {
    transform: scale(1);
    opacity: 1;
  }
}

/* Input Area */
.chat-input-area {
  padding: 20px;
  background: rgba(255, 255, 255, 0.95);
  border-top: 1px solid rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(10px);
}

.input-container {
  display: flex;
  gap: 12px;
  align-items: flex-end;
}

.message-input {
  flex: 1;
  min-height: 44px;
  max-height: 120px;
  padding: 10px 14px;
  border: 2px solid #e0e0e0;
  border-radius: 22px;
  font-family: inherit;
  font-size: 14px;
  line-height: 1.4;
  resize: none;
  background: white;
  transition: border-color 0.2s;
}

.message-input:focus {
  outline: none;
  border-color: #2196f3;
}

.message-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.send-button {
  width: 44px;
  height: 44px;
  border: none;
  border-radius: 22px;
  background: #2196f3;
  color: white;
  font-size: 16px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  flex-shrink: 0;
}

.send-button:hover:not(:disabled) {
  background: #1976d2;
  transform: scale(1.05);
}

.send-button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  transform: none;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.input-status {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 8px;
  font-size: 12px;
  color: #666;
}

.input-hints {
  display: flex;
  gap: 16px;
}

.hint {
  opacity: 0.7;
}

/* Error Message */
.error-message {
  position: fixed;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 1000;
}

.error-content {
  background: #f44336;
  color: white;
  padding: 12px 20px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 12px;
  box-shadow: 0 4px 16px rgba(244, 67, 54, 0.3);
  max-width: 400px;
}

.error-close {
  background: none;
  border: none;
  color: white;
  cursor: pointer;
  font-size: 16px;
  padding: 0;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .chat-window {
    background: linear-gradient(135deg, #2d3748 0%, #1a202c 100%);
  }

  .chat-header {
    background: rgba(45, 55, 72, 0.95);
    border-bottom-color: rgba(255, 255, 255, 0.1);
  }

  .chat-title {
    color: #e2e8f0;
  }

  .operation-badge {
    background: rgba(59, 130, 246, 0.2);
    color: #93c5fd;
  }

  .model-selector label {
    color: #cbd5e0;
  }

  .model-dropdown {
    background: #4a5568;
    color: #e2e8f0;
    border-color: #2d3748;
  }

  .toggle-label {
    color: #cbd5e0;
  }

  .action-btn {
    background: rgba(45, 55, 72, 0.8);
    color: #cbd5e0;
  }

  .message-assistant {
    background: rgba(45, 55, 72, 0.95);
    color: #e2e8f0;
  }

  .message-user {
    background: linear-gradient(135deg, rgba(59, 130, 246, 0.3) 0%, rgba(37, 99, 235, 0.4) 100%);
    color: #e2e8f0;
  }

  .message-role,
  .message-time {
    color: #a0aec0;
  }

  .user-content,
  .markdown-content {
    color: #e2e8f0;
  }

  .markdown-content :deep(pre) {
    background: #2d3748;
    border-color: #4a5568;
  }

  .markdown-content :deep(.inline-code) {
    background: #4a5568;
  }

  .typing-bubble {
    background: rgba(45, 55, 72, 0.9);
    color: #cbd5e0;
  }

  .chat-input-area {
    background: rgba(45, 55, 72, 0.95);
    border-top-color: rgba(255, 255, 255, 0.1);
  }

  .message-input {
    background: #4a5568;
    color: #e2e8f0;
    border-color: #2d3748;
  }

  .message-input:focus {
    border-color: #3182ce;
  }

  .character-count,
  .input-hints {
    color: #a0aec0;
  }

  .initial-text-preview {
    background: rgba(45, 55, 72, 0.8);
    border-left-color: #3182ce;
  }

  .initial-text {
    background: #2d3748;
    color: #e2e8f0;
  }

  .welcome-content {
    color: #cbd5e0;
  }

  .welcome-content h2 {
    color: #e2e8f0;
  }

  .thoughts-details {
    background: rgba(59, 130, 246, 0.1);
    border-color: rgba(59, 130, 246, 0.3);
  }

  .thoughts-header {
    background: rgba(59, 130, 246, 0.2);
    color: #93c5fd;
  }

  .thoughts-header:hover {
    background: rgba(59, 130, 246, 0.3);
  }

  .thoughts-markdown {
    color: white;
  }

  .thoughts-markdown :deep(code) {
    background: rgba(59, 130, 246, 0.2);
    color: #93c5fd;
  }

  .thoughts-markdown :deep(pre) {
    background: rgba(59, 130, 246, 0.15);
    border-color: rgba(59, 130, 246, 0.3);
  }

  /* Dark mode markdown headings */
  .markdown-content :deep(.markdown-h1) {
    color: #e2e8f0;
    border-bottom-color: #4a5568;
  }

  .markdown-content :deep(.markdown-h2) {
    color: #e2e8f0;
  }

  .markdown-content :deep(.markdown-h3) {
    color: #cbd5e0;
  }

  .markdown-content :deep(.markdown-h4) {
    color: #a0aec0;
  }

  .markdown-content :deep(.markdown-h5) {
    color: #a0aec0;
  }

  .markdown-content :deep(.markdown-h6) {
    color: #a0aec0;
  }

  /* Dark mode tables */
  .markdown-content :deep(.markdown-table) {
    border-color: #4a5568;
  }

  .markdown-content :deep(.markdown-table th) {
    background: #4a5568;
    color: #e2e8f0;
    border-bottom-color: #2d3748;
    border-right-color: #2d3748;
  }

  .markdown-content :deep(.markdown-table td) {
    border-bottom-color: #4a5568;
    border-right-color: #4a5568;
    color: #e2e8f0;
  }

  .markdown-content :deep(.markdown-table tr:nth-child(even)) {
    background: rgba(74, 85, 104, 0.3);
  }

  /* Dark mode blockquotes */
  .markdown-content :deep(.markdown-blockquote) {
    border-left-color: #3182ce;
    background: rgba(49, 130, 206, 0.1);
    color: #cbd5e0;
  }

  /* Dark mode horizontal rules */
  .markdown-content :deep(hr) {
    border-top-color: #4a5568;
  }
}

/* Thoughts Section */
.thoughts-section {
  margin-bottom: 12px;
}

.thoughts-details {
  background: rgba(173, 216, 230, 0.1); /* Light blue background */
  border: 1px solid rgba(173, 216, 230, 0.3);
  border-radius: 8px;
  overflow: hidden;
}

.thoughts-header {
  background: rgba(173, 216, 230, 0.2);
  padding: 8px 12px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  color: #4682b4; /* Steel blue color */
  display: flex;
  align-items: center;
  gap: 6px;
  user-select: none;
  border: none;
  outline: none;
}

.thoughts-header:hover {
  background: rgba(173, 216, 230, 0.3);
}

.thoughts-content {
  padding: 12px;
  font-size: 13px;
  line-height: 1.5;
}

.thoughts-markdown {
  color: white;
  font-style: italic;
}

.thoughts-markdown :deep(p) {
  margin: 6px 0;
}

.thoughts-markdown :deep(code) {
  background: rgba(173, 216, 230, 0.2);
  color: #4682b4;
}

.thoughts-markdown :deep(pre) {
  background: rgba(173, 216, 230, 0.15);
  border-color: rgba(173, 216, 230, 0.3);
}

/* Responsive */
@media (max-width: 768px) {
  .chat-header {
    flex-direction: column;
    gap: 12px;
    align-items: stretch;
  }

  .header-controls {
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 8px;
  }

  .message-bubble {
    max-width: 95%;
  }

  .input-container {
    gap: 8px;
  }

  .input-hints {
    display: none;
  }
}

@media (max-width: 480px) {
  .chat-messages {
    padding: 12px;
  }

  .chat-input-area {
    padding: 12px;
  }

  .message-bubble {
    padding: 12px;
  }

  .model-selector,
  .thinking-toggle {
    font-size: 12px;
  }

  .action-buttons {
    gap: 4px;
  }

  .action-btn {
    width: 28px;
    height: 28px;
    font-size: 12px;
  }
}
</style>