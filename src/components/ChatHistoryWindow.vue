<template>
  <div class="chat-history-window">
    <!-- Header -->
    <div class="history-header" data-tauri-drag-region>
      <div class="header-left" data-tauri-drag-region>
        <h1 class="history-title" data-tauri-drag-region>💬 Chat History</h1>
        <p class="history-subtitle" data-tauri-drag-region>Your AI text processing history</p>
      </div>
      <div class="header-controls">
        <button
          :disabled="isLoading"
          class="refresh-btn"
          title="Refresh history"
          data-tauri-drag-region="false"
          @click="refreshHistory"
        >
          🔄
        </button>
        <button
          :disabled="entries.length === 0 && conversations.length === 0"
          class="clear-btn"
          title="Clear all history"
          data-tauri-drag-region="false"
          @click="clearAllHistory"
        >
          🗑️
        </button>
        <button
          class="close-btn"
          title="Close window"
          data-tauri-drag-region="false"
          @click="closeWindow"
        >
          ✕
        </button>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="isLoading" class="loading-container">
      <LoadingSpinner :margin="true" />
      <p class="loading-text">Loading chat history...</p>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="error-container">
      <div class="error-icon">⚠️</div>
      <p class="error-message">{{ error }}</p>
      <button class="retry-button" @click="refreshHistory">Try Again</button>
    </div>

    <!-- Empty State -->
    <div v-else-if="entries.length === 0 && conversations.length === 0" class="empty-container">
      <div class="empty-icon">📝</div>
      <h3 class="empty-title">No history yet</h3>
      <p class="empty-message">
        Start using AI text operations to see your history here. Use Ctrl+Space to select text and
        process it with AI.
      </p>
    </div>

    <!-- History Content -->
    <div v-else class="history-content">
      <!-- Tab Navigation -->
      <div class="tab-navigation">
        <button
          :class="['tab-btn', { 'tab-btn--active': currentTab === 'conversations' }]"
          @click="currentTab = 'conversations'"
        >
          💬 Conversations ({{ conversations.length }})
        </button>
        <button
          :class="['tab-btn', { 'tab-btn--active': currentTab === 'entries' }]"
          @click="currentTab = 'entries'"
        >
          📝 Entries ({{ entries.length }})
        </button>
      </div>

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

      <!-- Conversations Tab -->
      <div v-if="currentTab === 'conversations'" class="history-entries">
        <div
          v-for="conversation in filteredConversations"
          :key="conversation.id"
          class="conversation-entry"
        >
          <div class="conversation-header">
            <div class="conversation-info">
              <span class="conversation-title">{{ conversation.title }}</span>
              <div class="conversation-meta">
                <span class="operation-badge" :class="getOperationClass(conversation.operation)">
                  {{ conversation.operation }}
                </span>
                <span class="conversation-timestamp">
                  {{ formatTimestamp(conversation.created_at) }}
                </span>
                <span class="conversation-message-count">
                  {{ conversation.messages.length }} messages
                </span>
              </div>
            </div>
            <div class="conversation-actions">
              <button
                class="reopen-btn"
                title="Continue this conversation"
                @click="reopenConversation(conversation)"
              >
                💬 Open
              </button>
              <button
                class="export-btn"
                title="Export as markdown"
                @click="exportConversation(conversation)"
              >
                📄 Export
              </button>
              <button
                class="delete-btn"
                title="Delete conversation"
                @click="deleteConversation(conversation.id)"
              >
                🗑️
              </button>
            </div>
          </div>

          <div class="conversation-preview">
            <div class="preview-messages">
              <div
                v-for="(message, idx) in conversation.messages.slice(0, 2)"
                :key="idx"
                class="preview-message"
                :class="message.role"
              >
                <span class="message-role">{{ message.role === 'user' ? '👤' : '🤖' }}:</span>
                <span class="message-text">
                  {{
                    message.content.length > 100
                      ? message.content.substring(0, 100) + '...'
                      : message.content
                  }}
                </span>
              </div>
              <div v-if="conversation.messages.length > 2" class="more-messages">
                +{{ conversation.messages.length - 2 }} more messages...
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Entries Tab -->
      <div v-else-if="currentTab === 'entries'" class="history-entries">
        <div v-for="(entry, index) in filteredEntries" :key="index" class="history-entry">
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
                class="copy-btn"
                title="Copy original text"
                @click="copyOriginalText(entry.original_text)"
              >
                📋 Original
              </button>
              <button
                class="copy-btn"
                title="Copy processed text"
                @click="copyProcessedText(entry.processed_text)"
              >
                📋 Result
              </button>
              <button
                class="reprocess-btn"
                title="Process again with same operation"
                @click="reprocessText(entry)"
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
              <SanitizedMarkdown
                class="text-content processed-text"
                :markdown="entry.processed_text"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- Stats Footer -->
      <div class="stats-footer">
        <div v-if="currentTab === 'conversations'" class="stats-info">
          Showing {{ filteredConversations.length }} of {{ conversations.length }} conversations
        </div>
        <div v-else class="stats-info">
          Showing {{ filteredEntries.length }} of {{ entries.length }} entries
        </div>
      </div>
    </div>

    <AppConfirmDialog
      :visible="confirmDialogVisible"
      :title="confirmDialogTitle"
      :message="confirmDialogMessage"
      :confirm-text="confirmDialogConfirmText"
      danger
      @confirm="handleConfirmDialogConfirm"
      @cancel="handleConfirmDialogCancel"
    />

    <AppToast :visible="toastVisible" :message="toastMessage" :type="toastType" />
  </div>
</template>

<script setup lang="ts">
  import { ref, computed, onMounted } from 'vue'
  import { invoke } from '@tauri-apps/api/core'
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import AppConfirmDialog from './AppConfirmDialog.vue'
  import AppToast from './AppToast.vue'
  import SanitizedMarkdown from './SanitizedMarkdown.vue'
  import LoadingSpinner from './LoadingSpinner.vue'
  import { logger } from '../utils/logger'
  import type { ChatEntry, SavedConversation } from '../types'
  import { useConfirmDialog } from '../composables/useConfirmDialog'
  import { useToast } from '../composables/useToast'

  // Reactive state
  const currentTab = ref<'conversations' | 'entries'>('conversations')
  const entries = ref<ChatEntry[]>([])
  const conversations = ref<SavedConversation[]>([])
  const isLoading = ref(true)
  const error = ref<string | null>(null)
  const searchQuery = ref('')
  const selectedOperation = ref('')

  // Dialog/toast state (via composables)
  const {
    visible: confirmDialogVisible,
    title: confirmDialogTitle,
    message: confirmDialogMessage,
    confirmText: confirmDialogConfirmText,
    open: openConfirmDialog,
    confirm: handleConfirmDialogConfirm,
    cancel: handleConfirmDialogCancel
  } = useConfirmDialog()
  const {
    visible: toastVisible,
    message: toastMessage,
    type: toastType,
    show: showToast
  } = useToast()

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
      filtered = filtered.filter(
        entry =>
          entry.original_text.toLowerCase().includes(query) ||
          entry.processed_text.toLowerCase().includes(query) ||
          entry.ai_option.toLowerCase().includes(query)
      )
    }

    // Filter by operation
    if (selectedOperation.value) {
      filtered = filtered.filter(entry => entry.ai_option === selectedOperation.value)
    }

    // Sort by timestamp (newest first) without mutating source array
    return [...filtered].sort(
      (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
    )
  })

  const filteredConversations = computed(() => {
    let filtered = conversations.value

    // Filter by search query
    if (searchQuery.value.trim()) {
      const query = searchQuery.value.toLowerCase()
      filtered = filtered.filter(
        conversation =>
          conversation.title.toLowerCase().includes(query) ||
          conversation.operation.toLowerCase().includes(query) ||
          conversation.messages.some(msg => msg.content.toLowerCase().includes(query))
      )
    }

    // Filter by operation
    if (selectedOperation.value) {
      filtered = filtered.filter(conversation => conversation.operation === selectedOperation.value)
    }

    // Sort by created_at timestamp (newest first) without mutating source array
    return [...filtered].sort(
      (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
    )
  })

  // Methods
  const loadHistory = async () => {
    try {
      isLoading.value = true
      error.value = null

      // Load both entries and conversations
      const [history, savedConversations] = await Promise.all([
        invoke('load_chat_history') as Promise<ChatEntry[]>,
        invoke('load_saved_conversations') as Promise<SavedConversation[]>
      ])

      entries.value = history
      conversations.value = savedConversations
    } catch (err) {
      logger.error('Failed to load history:', err)
      error.value = err instanceof Error ? err.message : 'Failed to load history'
    } finally {
      isLoading.value = false
    }
  }

  const refreshHistory = async () => {
    await loadHistory()
  }

  const clearAllHistory = async () => {
    const shouldClear = await openConfirmDialog({
      title: 'Clear All History',
      message: 'Are you sure you want to clear all chat history? This action cannot be undone.',
      confirmText: 'Clear All'
    })
    if (!shouldClear) {
      return
    }

    try {
      await Promise.all([invoke('clear_chat_history'), invoke('clear_saved_conversations')])

      entries.value = []
      conversations.value = []
      showToast('Chat history cleared', 'success')
    } catch (err) {
      logger.error('Failed to clear history:', err)
      error.value = 'Failed to clear history: ' + (err instanceof Error ? err.message : String(err))
      showToast(error.value, 'error')
    }
  }

  const copyOriginalText = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text)
      logger.debug('Original text copied to clipboard')
    } catch (err) {
      logger.error('Failed to copy text:', err)
    }
  }

  const copyProcessedText = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text)
      logger.debug('Processed text copied to clipboard')
    } catch (err) {
      logger.error('Failed to copy text:', err)
    }
  }

  const reprocessText = async (entry: ChatEntry) => {
    try {
      // Process the original text again with the same operation
      const result = (await invoke('process_text_with_ai', {
        text: entry.original_text,
        operation: entry.ai_option
      })) as string

      logger.debug('Text reprocessed successfully:', result)

      // Refresh history to show the new entry
      await loadHistory()
    } catch (err) {
      logger.error('Failed to reprocess text:', err)
      error.value =
        'Failed to reprocess text: ' + (err instanceof Error ? err.message : String(err))
    }
  }

  const closeWindow = async () => {
    try {
      const currentWindow = getCurrentWindow()
      await currentWindow.close()
    } catch (err) {
      logger.error('Failed to close window:', err)
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
        return (
          date.toLocaleDateString() +
          ' ' +
          date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
        )
      }
    } catch {
      return timestamp
    }
  }

  const getOperationClass = (operation: string): string => {
    const classMap: Record<string, string> = {
      Proofread: 'operation-proofread',
      Rewrite: 'operation-rewrite',
      Dansk: 'operation-translate',
      Concise: 'operation-concise',
      Friendly: 'operation-friendly',
      Professional: 'operation-professional',
      'Key Points': 'operation-keypoints',
      Summary: 'operation-summary',
      Chat: 'operation-chat',
      Custom: 'operation-custom'
    }
    return classMap[operation] || 'operation-default'
  }

  // New conversation management methods
  const reopenConversation = async (conversation: SavedConversation) => {
    try {
      // Use backend command to create chat window (more reliable than frontend WebviewWindow)
      await invoke('reopen_chat_conversation', {
        conversationId: conversation.id,
        operation: conversation.operation,
        title: conversation.title
      })

      logger.debug('Reopened conversation:', conversation.title)
    } catch (err) {
      logger.error('Failed to reopen conversation:', err)
      error.value =
        'Failed to reopen conversation: ' + (err instanceof Error ? err.message : String(err))
      showToast('Failed to reopen conversation', 'error')
    }
  }

  const exportConversation = async (conversation: SavedConversation) => {
    try {
      // Generate markdown content
      let markdown = `# ${conversation.title}\n\n`
      markdown += `**Operation:** ${conversation.operation}  \n`
      markdown += `**Created:** ${new Date(conversation.created_at).toLocaleString()}  \n`
      markdown += `**Messages:** ${conversation.messages.length}  \n\n`
      markdown += `---\n\n`

      conversation.messages.forEach((message, _index) => {
        const role = message.role === 'user' ? '👤 **User**' : '🤖 **Assistant**'
        markdown += `## ${role}\n\n${message.content}\n\n`
        if (message.sources?.length) {
          markdown += `### Sources\n\n`
          message.sources.forEach(source => {
            markdown += `- [${source.title}](${source.uri})\n`
          })
          markdown += `\n`
        }
      })

      // Copy to clipboard
      await navigator.clipboard.writeText(markdown)
      showToast('Conversation exported to clipboard as Markdown', 'success')
    } catch (err) {
      logger.error('Failed to export conversation:', err)
      error.value = 'Failed to export conversation'
      showToast('Failed to export conversation', 'error')
    }
  }

  const deleteConversation = async (conversationId: string) => {
    const conversation = conversations.value.find(c => c.id === conversationId)
    if (!conversation) return

    const confirmDelete = await openConfirmDialog({
      title: 'Delete Conversation',
      message: `Are you sure you want to delete the conversation "${conversation.title}"?\n\nThis action cannot be undone.`,
      confirmText: 'Delete'
    })
    if (!confirmDelete) return

    try {
      await invoke('delete_saved_conversation', { conversationId })

      // Remove from local state
      conversations.value = conversations.value.filter(c => c.id !== conversationId)

      logger.debug('Conversation deleted:', conversation.title)
    } catch (err) {
      logger.error('Failed to delete conversation:', err)
      error.value =
        'Failed to delete conversation: ' + (err instanceof Error ? err.message : String(err))
      showToast('Failed to delete conversation', 'error')
    }
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
    background: linear-gradient(135deg, var(--color-bg-surface) 0%, var(--color-bg-app) 100%);
    font-family: var(--font-family-base);
  }

  /* Header */
  .history-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-5) var(--space-6) var(--space-4) var(--space-6);
    background: var(--color-bg-surface);
    border-bottom: 1px solid var(--color-border-subtle);
    backdrop-filter: blur(10px);
  }

  .header-left {
    flex: 1;
  }

  .history-title {
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-bold);
    color: var(--color-text-primary);
    margin: 0 0 var(--space-1) 0;
  }

  .history-subtitle {
    font-size: var(--font-size-base);
    color: var(--color-text-tertiary);
    margin: 0;
  }

  .header-controls {
    display: flex;
    gap: var(--space-2);
  }

  .refresh-btn,
  .clear-btn,
  .close-btn {
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: var(--space-2) var(--space-3);
    cursor: pointer;
    font-size: var(--font-size-base);
    color: var(--color-text-primary);
    transition: all var(--transition-base);
  }

  .refresh-btn:hover:not(:disabled),
  .clear-btn:hover:not(:disabled) {
    background: var(--color-border);
  }

  .close-btn:hover {
    background: var(--color-danger);
    color: white;
    border-color: var(--color-danger);
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
    padding: var(--space-10);
  }

  .loading-text {
    color: var(--color-text-tertiary);
    font-size: var(--font-size-base);
    margin: 0;
  }

  .error-container {
    color: var(--color-danger);
  }

  .error-icon {
    font-size: 32px;
    margin-bottom: var(--space-3);
  }

  .error-message {
    margin: var(--space-2) 0 var(--space-4) 0;
    font-size: var(--font-size-base);
  }

  .retry-button {
    background: var(--color-accent);
    color: white;
    border: none;
    padding: var(--space-3) var(--space-5);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-base);
    transition: background var(--transition-base);
  }

  .retry-button:hover {
    background: var(--color-accent-hover);
  }

  .empty-icon {
    font-size: 64px;
    margin-bottom: var(--space-4);
  }

  .empty-title {
    font-size: var(--font-size-lg);
    color: var(--color-text-primary);
    margin: 0 0 var(--space-2) 0;
  }

  .empty-message {
    color: var(--color-text-tertiary);
    font-size: var(--font-size-base);
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
    padding: var(--space-4) var(--space-6);
    background: var(--color-bg-surface);
    border-bottom: 1px solid var(--color-border-subtle);
    display: flex;
    gap: var(--space-4);
    align-items: center;
  }

  .search-box {
    position: relative;
    flex: 1;
    max-width: 400px;
  }

  .search-input {
    width: 100%;
    padding: 10px var(--space-4) 10px 40px;
    border: 1px solid var(--input-border);
    border-radius: var(--radius-pill);
    font-size: var(--font-size-base);
    background: var(--input-bg);
    color: var(--input-text);
    transition:
      border-color var(--transition-base),
      box-shadow var(--transition-base);
  }

  .search-input:focus {
    outline: none;
    border-color: var(--input-border-focus);
    box-shadow: var(--shadow-focus);
  }

  .search-icon {
    position: absolute;
    left: 12px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--color-text-muted);
    font-size: 14px;
  }

  .operation-filter {
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-base);
    background: var(--input-bg);
    color: var(--input-text);
    min-width: 150px;
  }

  .history-entries {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-4) var(--space-6);
  }

  .history-entry {
    background: var(--color-bg-surface);
    border-radius: var(--radius-lg);
    margin-bottom: var(--space-4);
    box-shadow: var(--shadow-sm);
    overflow: hidden;
    transition: transform var(--transition-base);
  }

  .history-entry:hover {
    transform: translateY(-1px);
    box-shadow: var(--shadow-md);
  }

  .entry-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-4) var(--space-5);
    background: var(--color-bg-elevated);
    border-bottom: 1px solid var(--color-border);
  }

  .entry-info {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .operation-badge {
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    color: white;
  }

  .operation-proofread {
    background: var(--color-success);
  }
  .operation-rewrite {
    background: var(--color-accent);
  }
  .operation-translate {
    background: var(--color-warning);
  }
  .operation-concise {
    background: #9c27b0;
  }
  .operation-friendly {
    background: #e91e63;
  }
  .operation-professional {
    background: var(--color-text-muted);
  }
  .operation-keypoints {
    background: #795548;
  }
  .operation-summary {
    background: #00bcd4;
  }
  .operation-chat {
    background: #3f51b5;
  }
  .operation-custom {
    background: #ff5722;
  }
  .operation-default {
    background: #757575;
  }

  .entry-timestamp {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
  }

  .entry-actions {
    display: flex;
    gap: var(--space-2);
  }

  .copy-btn,
  .reprocess-btn {
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 6px 10px;
    cursor: pointer;
    font-size: 11px;
    color: var(--color-text-primary);
    transition: all var(--transition-base);
  }

  .copy-btn:hover {
    background: var(--color-border);
  }

  .reprocess-btn:hover {
    background: var(--color-accent-soft);
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .entry-content {
    padding: var(--space-5);
  }

  .text-section {
    margin-bottom: var(--space-4);
  }

  .text-section:last-child {
    margin-bottom: 0;
  }

  .text-label {
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: var(--space-2);
  }

  .text-content {
    line-height: 1.6;
    font-size: var(--font-size-base);
  }

  .original-text {
    color: var(--color-text-primary);
    background: var(--color-bg-elevated);
    padding: var(--space-3);
    border-radius: var(--radius-sm);
    border-left: 3px solid var(--color-accent);
  }

  .processed-text {
    color: var(--color-text-primary);
  }

  /* Markdown Headings */
  .processed-text :deep(.markdown-heading) {
    margin: var(--space-4) 0 var(--space-2) 0;
    font-weight: var(--font-weight-semibold);
    line-height: 1.2;
  }

  .processed-text :deep(.markdown-h1) {
    font-size: 20px;
    color: var(--color-text-primary);
    border-bottom: 2px solid var(--color-border);
    padding-bottom: 6px;
  }

  .processed-text :deep(.markdown-h2) {
    font-size: var(--font-size-lg);
    color: var(--color-text-primary);
  }

  .processed-text :deep(.markdown-h3) {
    font-size: var(--font-size-md);
    color: var(--color-text-secondary);
  }

  .processed-text :deep(.markdown-h4) {
    font-size: var(--font-size-base);
    color: var(--color-text-secondary);
  }

  .processed-text :deep(.markdown-h5) {
    font-size: var(--font-size-sm);
    color: var(--color-text-tertiary);
  }

  .processed-text :deep(.markdown-h6) {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
  }

  /* Markdown Tables */
  .processed-text :deep(.table-wrapper) {
    overflow-x: auto;
    margin: var(--space-3) 0;
  }

  .processed-text :deep(.markdown-table) {
    width: 100%;
    border-collapse: collapse;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
    font-size: 13px;
  }

  .processed-text :deep(.markdown-table th) {
    background: var(--color-bg-elevated);
    padding: 6px 10px;
    text-align: left;
    font-weight: var(--font-weight-semibold);
    border-bottom: 1px solid var(--color-border);
    border-right: 1px solid var(--color-border);
  }

  .processed-text :deep(.markdown-table th:last-child) {
    border-right: none;
  }

  .processed-text :deep(.markdown-table td) {
    padding: 6px 10px;
    border-bottom: 1px solid var(--color-border);
    border-right: 1px solid var(--color-border);
  }

  .processed-text :deep(.markdown-table td:last-child) {
    border-right: none;
  }

  .processed-text :deep(.markdown-table tr:last-child td) {
    border-bottom: none;
  }

  .processed-text :deep(.markdown-table tr:nth-child(even)) {
    background: var(--color-bg-elevated);
  }

  /* Markdown Blockquotes */
  .processed-text :deep(.markdown-blockquote) {
    margin: var(--space-3) 0;
    padding: 10px var(--space-4);
    border-left: 3px solid var(--color-accent);
    background: var(--color-accent-soft);
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    color: var(--color-text-secondary);
    font-style: italic;
    font-size: 13px;
  }

  .processed-text :deep(.markdown-blockquote p) {
    margin: 0;
  }

  /* Markdown Lists */
  .processed-text :deep(.markdown-list) {
    margin: var(--space-2) 0;
    padding-left: 20px;
    font-size: 13px;
  }

  .processed-text :deep(.markdown-list li) {
    margin: 2px 0;
    line-height: 1.4;
  }

  /* Horizontal Rules */
  .processed-text :deep(hr) {
    margin: var(--space-3) 0;
    border: none;
    border-top: 1px solid var(--color-border);
  }

  /* Copy button for code blocks */
  .processed-text :deep(.copy-code-btn) {
    position: absolute;
    top: 6px;
    right: 6px;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: 3px;
    padding: 2px 6px;
    font-size: 11px;
    cursor: pointer;
    color: var(--color-text-primary);
  }

  .processed-text :deep(.code-block) {
    position: relative;
  }

  .stats-footer {
    padding: var(--space-4) var(--space-6);
    background: var(--color-bg-surface);
    border-top: 1px solid var(--color-border-subtle);
    text-align: center;
  }

  .stats-info {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
  }

  /* Tabs */
  .tab-navigation {
    display: flex;
    background: var(--color-bg-surface);
    border-bottom: 1px solid var(--color-border-subtle);
    padding: var(--space-4) var(--space-6) 0 var(--space-6);
  }

  .tab-btn {
    background: transparent;
    border: none;
    border-bottom: 3px solid transparent;
    padding: var(--space-3) var(--space-4);
    cursor: pointer;
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-medium);
    color: var(--color-text-tertiary);
    transition: all var(--transition-base);
    margin-right: var(--space-4);
  }

  .tab-btn:hover {
    color: var(--color-accent);
    background: var(--color-accent-soft);
  }

  .tab-btn--active {
    color: var(--color-accent);
    border-bottom-color: var(--color-accent);
    background: var(--color-accent-soft);
  }

  .conversation-entry {
    background: var(--color-bg-surface);
    border-radius: var(--radius-lg);
    margin-bottom: var(--space-4);
    box-shadow: var(--shadow-sm);
    overflow: hidden;
    transition:
      transform var(--transition-base),
      box-shadow var(--transition-base);
    cursor: pointer;
  }

  .conversation-entry:hover {
    transform: translateY(-1px);
    box-shadow: var(--shadow-md);
  }

  .conversation-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: var(--space-5);
    background: var(--color-bg-elevated);
    border-bottom: 1px solid var(--color-border);
  }

  .conversation-info {
    flex: 1;
  }

  .conversation-title {
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-primary);
    margin-bottom: var(--space-2);
    display: block;
    line-height: 1.3;
  }

  .conversation-meta {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex-wrap: wrap;
  }

  .conversation-timestamp {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
  }

  .conversation-message-count {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    background: var(--color-border-subtle);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }

  .conversation-actions {
    display: flex;
    gap: var(--space-2);
    flex-shrink: 0;
  }

  .reopen-btn,
  .export-btn,
  .delete-btn {
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 6px 10px;
    cursor: pointer;
    font-size: 11px;
    color: var(--color-text-primary);
    transition: all var(--transition-base);
    white-space: nowrap;
  }

  .reopen-btn:hover {
    background: var(--color-accent-soft);
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .export-btn:hover {
    background: rgba(156, 39, 176, 0.15);
    border-color: #9c27b0;
    color: #ce93d8;
  }

  .delete-btn:hover {
    background: var(--color-danger-soft);
    border-color: var(--color-danger);
    color: var(--color-danger);
  }

  .conversation-preview {
    padding: var(--space-4) var(--space-5);
  }

  .preview-messages {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .preview-message {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    font-size: 13px;
    line-height: 1.4;
  }

  .preview-message.user {
    color: var(--color-text-primary);
  }

  .preview-message.assistant {
    color: var(--color-text-secondary);
    background: var(--color-accent-soft);
    padding: var(--space-2);
    border-radius: var(--radius-sm);
    margin-left: var(--space-3);
  }

  .message-role {
    font-size: var(--font-size-xs);
    flex-shrink: 0;
  }

  .message-text {
    flex: 1;
    word-break: break-word;
  }

  .more-messages {
    font-size: var(--font-size-xs);
    color: var(--color-text-muted);
    font-style: italic;
    margin-top: var(--space-1);
    text-align: center;
  }

  /* Responsive */
  @media (max-width: 768px) {
    .history-header {
      padding: var(--space-4);
      flex-direction: column;
      align-items: flex-start;
      gap: var(--space-3);
    }

    .header-controls {
      align-self: flex-end;
    }

    .search-section {
      flex-direction: column;
      gap: var(--space-3);
      align-items: stretch;
    }

    .search-box {
      max-width: none;
    }

    .history-entries {
      padding: var(--space-3) var(--space-4);
    }

    .entry-header {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--space-3);
    }

    .entry-actions {
      align-self: flex-end;
    }

    .entry-content {
      padding: var(--space-4);
    }

    .tab-navigation {
      padding: var(--space-3) var(--space-4) 0 var(--space-4);
    }

    .tab-btn {
      padding: var(--space-2) var(--space-3);
      font-size: 13px;
      margin-right: var(--space-2);
    }

    .conversation-header {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--space-3);
      padding: var(--space-4);
    }

    .conversation-actions {
      align-self: flex-end;
    }

    .conversation-meta {
      flex-wrap: wrap;
      gap: var(--space-2);
    }

    .conversation-preview {
      padding: var(--space-3) var(--space-4);
    }

    .preview-message.assistant {
      margin-left: var(--space-2);
    }
  }
</style>
