<template>
  <div class="chat-history-window">
    <!-- Header -->
    <div class="history-header">
      <div class="header-left">
        <h1 class="history-title">💬 Chat History</h1>
        <p class="history-subtitle">Your AI text processing history</p>
      </div>
      <div class="header-controls">
        <button 
          @click="refreshHistory" 
          :disabled="isLoading"
          class="refresh-btn"
          title="Refresh history"
        >
          🔄
        </button>
        <button 
          @click="clearAllHistory" 
          :disabled="entries.length === 0"
          class="clear-btn"
          title="Clear all history"
        >
          🗑️
        </button>
        <button 
          @click="closeWindow" 
          class="close-btn"
          title="Close window"
        >
          ✕
        </button>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="isLoading" class="loading-container">
      <div class="loading-spinner"></div>
      <p class="loading-text">Loading chat history...</p>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="error-container">
      <div class="error-icon">⚠️</div>
      <p class="error-message">{{ error }}</p>
      <button @click="refreshHistory" class="retry-button">Try Again</button>
    </div>

    <!-- Empty State -->
    <div v-else-if="entries.length === 0" class="empty-container">
      <div class="empty-icon">📝</div>
      <h3 class="empty-title">No history yet</h3>
      <p class="empty-message">
        Start using AI text operations to see your history here. 
        Use Ctrl+Space to select text and process it with AI.
      </p>
    </div>

    <!-- History Content -->
    <div v-else class="history-content">
      <!-- Search and Filter -->
      <div class="search-section">
        <div class="search-box">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search history..."
            class="search-input"
          />
          <span class="search-icon">🔍</span>
        </div>
        <div class="filter-section">
          <select v-model="selectedOperation" class="operation-filter">
            <option value="">All operations</option>
            <option v-for="operation in uniqueOperations" :key="operation" :value="operation">
              {{ operation }}
            </option>
          </select>
        </div>
      </div>

      <!-- History Entries -->
      <div class="history-entries">
        <div 
          v-for="(entry, index) in filteredEntries" 
          :key="index" 
          class="history-entry"
        >
          <div class="entry-header">
            <div class="entry-info">
              <span class="operation-badge" :class="getOperationClass(entry.ai_option)">
                {{ entry.ai_option }}
              </span>
              <span class="entry-timestamp">
                {{ formatTimestamp(entry.timestamp) }}
              </span>
            </div>
            <div class="entry-actions">
              <button 
                @click="copyOriginalText(entry.original_text)"
                class="copy-btn"
                title="Copy original text"
              >
                📋 Original
              </button>
              <button 
                @click="copyProcessedText(entry.processed_text)"
                class="copy-btn"
                title="Copy processed text"
              >
                📋 Result
              </button>
              <button 
                @click="reprocessText(entry)"
                class="reprocess-btn"
                title="Process again with same operation"
              >
                🔄 Reprocess
              </button>
            </div>
          </div>
          
          <div class="entry-content">
            <div class="text-section">
              <div class="text-label">Original Text:</div>
              <div class="text-content original-text">
                {{ entry.original_text }}
              </div>
            </div>
            
            <div class="text-section">
              <div class="text-label">AI Result:</div>
              <div class="text-content processed-text" v-html="renderMarkdown(entry.processed_text)">
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Stats Footer -->
      <div class="stats-footer">
        <div class="stats-info">
          Showing {{ filteredEntries.length }} of {{ entries.length }} entries
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'

// Chat Entry Interface
interface ChatEntry {
  timestamp: string
  original_text: string
  ai_option: string
  processed_text: string
}

// Reactive state
const entries = ref<ChatEntry[]>([])
const isLoading = ref(true)
const error = ref<string | null>(null)
const searchQuery = ref('')
const selectedOperation = ref('')

// Computed properties
const uniqueOperations = computed(() => {
  const operations = entries.value.map(entry => entry.ai_option)
  return [...new Set(operations)].sort()
})

const filteredEntries = computed(() => {
  let filtered = entries.value

  // Filter by search query
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    filtered = filtered.filter(entry => 
      entry.original_text.toLowerCase().includes(query) ||
      entry.processed_text.toLowerCase().includes(query) ||
      entry.ai_option.toLowerCase().includes(query)
    )
  }

  // Filter by operation
  if (selectedOperation.value) {
    filtered = filtered.filter(entry => entry.ai_option === selectedOperation.value)
  }

  // Sort by timestamp (newest first)
  return filtered.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
})

// Methods
const loadHistory = async () => {
  try {
    isLoading.value = true
    error.value = null
    
    const history = await invoke('load_chat_history') as ChatEntry[]
    entries.value = history
    
  } catch (err) {
    console.error('Failed to load chat history:', err)
    error.value = err instanceof Error ? err.message : 'Failed to load chat history'
  } finally {
    isLoading.value = false
  }
}

const refreshHistory = async () => {
  await loadHistory()
}

const clearAllHistory = async () => {
  if (!confirm('Are you sure you want to clear all chat history? This action cannot be undone.')) {
    return
  }

  try {
    // Clear the history using the dedicated command
    await invoke('clear_chat_history')
    
    // Clear local state to show empty state immediately
    entries.value = []
    
  } catch (err) {
    console.error('Failed to clear history:', err)
    error.value = 'Failed to clear history: ' + (err instanceof Error ? err.message : String(err))
  }
}

const copyOriginalText = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    console.log('Original text copied to clipboard')
  } catch (err) {
    console.error('Failed to copy text:', err)
  }
}

const copyProcessedText = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    console.log('Processed text copied to clipboard')
  } catch (err) {
    console.error('Failed to copy text:', err)
  }
}

const reprocessText = async (entry: ChatEntry) => {
  try {
    // Process the original text again with the same operation
    const result = await invoke('process_text_with_ai', {
      text: entry.original_text,
      operation: entry.ai_option
    }) as string
    
    console.log('Text reprocessed successfully:', result)
    
    // Refresh history to show the new entry
    await loadHistory()
    
  } catch (err) {
    console.error('Failed to reprocess text:', err)
    error.value = 'Failed to reprocess text: ' + (err instanceof Error ? err.message : String(err))
  }
}

const closeWindow = async () => {
  try {
    const currentWindow = getCurrentWindow()
    await currentWindow.close()
  } catch (err) {
    console.error('Failed to close window:', err)
  }
}

const formatTimestamp = (timestamp: string): string => {
  try {
    const date = new Date(timestamp)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffHours = diffMs / (1000 * 60 * 60)
    const diffDays = diffMs / (1000 * 60 * 60 * 24)
    
    if (diffHours < 1) {
      const diffMins = Math.floor(diffMs / (1000 * 60))
      return `${diffMins} min${diffMins !== 1 ? 's' : ''} ago`
    } else if (diffHours < 24) {
      const hours = Math.floor(diffHours)
      return `${hours} hour${hours !== 1 ? 's' : ''} ago`
    } else if (diffDays < 7) {
      const days = Math.floor(diffDays)
      return `${days} day${days !== 1 ? 's' : ''} ago`
    } else {
      return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    }
  } catch {
    return timestamp
  }
}

const getOperationClass = (operation: string): string => {
  const classMap: Record<string, string> = {
    'Proofread': 'operation-proofread',
    'Rewrite': 'operation-rewrite', 
    'Dansk': 'operation-translate',
    'Concise': 'operation-concise',
    'Friendly': 'operation-friendly',
    'Professional': 'operation-professional',
    'Key Points': 'operation-keypoints',
    'Summary': 'operation-summary',
    'Chat': 'operation-chat',
    'Custom': 'operation-custom'
  }
  return classMap[operation] || 'operation-default'
}

const renderMarkdown = (text: string): string => {
  let html = text
  
  // Code blocks
  html = html.replace(/```(\w+)?\n([\s\S]*?)```/g, (match, lang, code) => {
    const language = lang || 'text'
    return `<pre class="code-block"><code class="language-${language}">${escapeHtml(code.trim())}</code></pre>`
  })
  
  // Inline code
  html = html.replace(/`([^`]+)`/g, '<code class="inline-code">$1</code>')
  
  // Bold
  html = html.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
  
  // Italic
  html = html.replace(/\*([^*]+)\*/g, '<em>$1</em>')
  
  // Line breaks
  html = html.replace(/\n/g, '<br>')
  
  return html
}

const escapeHtml = (text: string): string => {
  const div = document.createElement('div')
  div.textContent = text
  return div.innerHTML
}

// Lifecycle
onMounted(async () => {
  await loadHistory()
})
</script>

<style scoped>
.chat-history-window {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

/* Header */
.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px 16px 24px;
  background: rgba(255, 255, 255, 0.95);
  border-bottom: 1px solid rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(10px);
}

.header-left {
  flex: 1;
}

.history-title {
  font-size: 24px;
  font-weight: 700;
  color: #333;
  margin: 0 0 4px 0;
}

.history-subtitle {
  font-size: 14px;
  color: #666;
  margin: 0;
}

.header-controls {
  display: flex;
  gap: 8px;
}

.refresh-btn,
.clear-btn,
.close-btn {
  background: white;
  border: 1px solid #ddd;
  border-radius: 6px;
  padding: 8px 12px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.refresh-btn:hover,
.clear-btn:hover {
  background: #f5f5f5;
  border-color: #999;
}

.close-btn:hover {
  background: #ff4444;
  color: white;
  border-color: #ff4444;
}

.refresh-btn:disabled,
.clear-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* Loading, Error, Empty States */
.loading-container,
.error-container,
.empty-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  text-align: center;
  padding: 40px;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #f3f3f3;
  border-top: 3px solid #2196F3;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 16px;
}

.loading-text {
  color: #666;
  font-size: 14px;
  margin: 0;
}

.error-container {
  color: #d32f2f;
}

.error-icon {
  font-size: 32px;
  margin-bottom: 12px;
}

.error-message {
  margin: 8px 0 16px 0;
  font-size: 14px;
}

.retry-button {
  background: #2196F3;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
}

.empty-icon {
  font-size: 64px;
  margin-bottom: 16px;
}

.empty-title {
  font-size: 18px;
  color: #333;
  margin: 0 0 8px 0;
}

.empty-message {
  color: #666;
  font-size: 14px;
  line-height: 1.5;
  margin: 0;
  max-width: 400px;
}

/* History Content */
.history-content {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
}

.search-section {
  padding: 16px 24px;
  background: rgba(255, 255, 255, 0.8);
  border-bottom: 1px solid rgba(0, 0, 0, 0.1);
  display: flex;
  gap: 16px;
  align-items: center;
}

.search-box {
  position: relative;
  flex: 1;
  max-width: 400px;
}

.search-input {
  width: 100%;
  padding: 10px 16px 10px 40px;
  border: 1px solid #ddd;
  border-radius: 20px;
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.search-input:focus {
  border-color: #2196F3;
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  color: #666;
  font-size: 14px;
}

.operation-filter {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
  background: white;
  min-width: 150px;
}

.history-entries {
  flex: 1;
  overflow-y: auto;
  padding: 16px 24px;
}

.history-entry {
  background: white;
  border-radius: 12px;
  margin-bottom: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  overflow: hidden;
  transition: transform 0.2s;
}

.history-entry:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
}

.entry-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: #f8f9fa;
  border-bottom: 1px solid #e9ecef;
}

.entry-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.operation-badge {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
  color: white;
}

.operation-proofread { background: #4caf50; }
.operation-rewrite { background: #2196F3; }
.operation-translate { background: #ff9800; }
.operation-concise { background: #9c27b0; }
.operation-friendly { background: #e91e63; }
.operation-professional { background: #607d8b; }
.operation-keypoints { background: #795548; }
.operation-summary { background: #00bcd4; }
.operation-chat { background: #3f51b5; }
.operation-custom { background: #ff5722; }
.operation-default { background: #757575; }

.entry-timestamp {
  font-size: 12px;
  color: #666;
}

.entry-actions {
  display: flex;
  gap: 8px;
}

.copy-btn,
.reprocess-btn {
  background: white;
  border: 1px solid #ddd;
  border-radius: 4px;
  padding: 6px 10px;
  cursor: pointer;
  font-size: 11px;
  transition: all 0.2s;
}

.copy-btn:hover {
  background: #f0f0f0;
  border-color: #999;
}

.reprocess-btn:hover {
  background: #e3f2fd;
  border-color: #2196F3;
  color: #2196F3;
}

.entry-content {
  padding: 20px;
}

.text-section {
  margin-bottom: 16px;
}

.text-section:last-child {
  margin-bottom: 0;
}

.text-label {
  font-size: 12px;
  font-weight: 600;
  color: #666;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 8px;
}

.text-content {
  line-height: 1.6;
  font-size: 14px;
}

.original-text {
  color: #333;
  background: #f8f9fa;
  padding: 12px;
  border-radius: 6px;
  border-left: 3px solid #2196F3;
}

.processed-text {
  color: #333;
}

.processed-text .code-block {
  background: #f4f4f4;
  padding: 12px;
  border-radius: 4px;
  margin: 8px 0;
  overflow-x: auto;
}

.processed-text .inline-code {
  background: #f4f4f4;
  padding: 2px 4px;
  border-radius: 3px;
  font-family: 'Monaco', 'Consolas', monospace;
  font-size: 13px;
}

.stats-footer {
  padding: 16px 24px;
  background: rgba(255, 255, 255, 0.8);
  border-top: 1px solid rgba(0, 0, 0, 0.1);
  text-align: center;
}

.stats-info {
  font-size: 12px;
  color: #666;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .chat-history-window {
    background: linear-gradient(135deg, #2d3748 0%, #1a202c 100%);
  }

  .history-header {
    background: rgba(45, 55, 72, 0.95);
    border-bottom-color: rgba(255, 255, 255, 0.1);
  }

  .history-title {
    color: #e2e8f0;
  }

  .history-subtitle {
    color: #a0aec0;
  }

  .refresh-btn,
  .clear-btn,
  .close-btn {
    background: #4a5568;
    border-color: #2d3748;
    color: #e2e8f0;
  }

  .search-section {
    background: rgba(45, 55, 72, 0.8);
    border-bottom-color: rgba(255, 255, 255, 0.1);
  }

  .search-input {
    background: #4a5568;
    border-color: #2d3748;
    color: #e2e8f0;
  }

  .operation-filter {
    background: #4a5568;
    border-color: #2d3748;
    color: #e2e8f0;
  }

  .history-entry {
    background: rgba(45, 55, 72, 0.8);
  }

  .entry-header {
    background: #2d3748;
    border-bottom-color: #4a5568;
  }

  .entry-timestamp {
    color: #a0aec0;
  }

  .copy-btn,
  .reprocess-btn {
    background: #4a5568;
    border-color: #2d3748;
    color: #e2e8f0;
  }

  .text-label {
    color: #a0aec0;
  }

  .original-text {
    background: #2d3748;
    color: #e2e8f0;
    border-left-color: #3182ce;
  }

  .processed-text {
    color: #e2e8f0;
  }

  .stats-footer {
    background: rgba(45, 55, 72, 0.8);
    border-top-color: rgba(255, 255, 255, 0.1);
  }

  .stats-info {
    color: #a0aec0;
  }
}

/* Responsive */
@media (max-width: 768px) {
  .history-header {
    padding: 16px 16px 12px 16px;
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .header-controls {
    align-self: flex-end;
  }

  .search-section {
    flex-direction: column;
    gap: 12px;
    align-items: stretch;
  }

  .search-box {
    max-width: none;
  }

  .history-entries {
    padding: 12px 16px;
  }

  .entry-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .entry-actions {
    align-self: flex-end;
  }

  .entry-content {
    padding: 16px;
  }
}
</style>