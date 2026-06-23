<template>
  <div class="chat-window" @keydown="handleGlobalKeydown">
    <!-- Header -->
    <div class="chat-header" data-tauri-drag-region>
      <div class="header-left" data-tauri-drag-region>
        <h1 class="chat-title" data-tauri-drag-region>{{ windowTitle }}</h1>
        <div v-if="showOperationBadge" class="operation-info" data-tauri-drag-region>
          <span class="operation-badge" data-tauri-drag-region>{{ operation }}</span>
        </div>
      </div>

      <div class="header-center" data-tauri-drag-region>
        <div class="model-selector">
          <select
            id="model-select"
            v-model="state.selectedModel"
            class="model-dropdown"
            :disabled="state.isProcessing"
            data-tauri-drag-region="false"
          >
            <option v-for="model in state.availableModels" :key="model" :value="model">
              {{ formatModelName(model) }}
            </option>
          </select>
        </div>

        <div v-if="supportsThinking" class="thinking-toggle">
          <label
            class="toggle-label toggle-label--icon"
            title="Thinking mode gives Gemini more room to reason before answering."
          >
            <span class="thinking-icon" aria-hidden="true">
              <AppIcon :icon="Brain" :size="16" />
            </span>
            <input
              v-model="state.enableThinking"
              type="checkbox"
              :disabled="state.isProcessing"
              data-tauri-drag-region="false"
            />
            <span class="toggle-slider"></span>
          </label>
        </div>

        <div v-if="supportsGrounding" class="thinking-toggle">
          <label
            class="toggle-label toggle-label--icon"
            title="Let Gemini use Google Search for fresher, source-backed answers."
          >
            <span class="thinking-icon" aria-hidden="true">
              <AppIcon :icon="Search" :size="16" />
            </span>
            <input
              v-model="state.enableGrounding"
              type="checkbox"
              :disabled="state.isProcessing"
              data-tauri-drag-region="false"
            />
            <span class="toggle-slider"></span>
          </label>
        </div>
      </div>

      <div class="header-actions">
        <div class="action-buttons">
          <button
            class="action-btn save-btn"
            :disabled="state.messages.length === 0"
            title="Save conversation to history"
            data-tauri-drag-region="false"
            @click="saveConversation"
          >
            <AppIcon :icon="Save" :size="16" />
          </button>
          <button
            class="action-btn clear-btn"
            :disabled="state.messages.length === 0"
            title="Clear conversation (Ctrl+L)"
            data-tauri-drag-region="false"
            @click="clearConversation"
          >
            <AppIcon :icon="Trash2" :size="16" />
          </button>
          <button
            class="action-btn close-btn"
            title="Close window (Escape)"
            data-tauri-drag-region="false"
            @click="closeWindow"
          >
            <AppIcon :icon="X" :size="16" />
          </button>
        </div>
      </div>
    </div>

    <!-- Chat Messages Area -->
    <div ref="messagesContainer" class="chat-messages">
      <!-- Welcome Message -->
      <div v-if="state.messages.length === 0" class="welcome-message">
        <div class="welcome-content">
          <h2 class="welcome-heading">
            <AppIcon :icon="MessageSquareText" :size="20" />
            AI Chat Assistant
          </h2>
          <p v-if="initialText">Ready to process your text and answer follow-up questions.</p>
          <p v-else>Start a conversation by typing your message below.</p>
          <div v-if="initialText" class="initial-text-preview">
            <strong>Selected text:</strong>
            <div class="initial-text">{{ initialTextPreview }}</div>
          </div>
        </div>
      </div>

      <!-- Message Bubbles -->
      <MessageBubble
        v-for="(message, index) in state.messages"
        :key="`msg-${index}`"
        :message="message"
        @regenerate="regenerateResponse(index)"
      />

      <!-- Typing Indicator -->
      <div
        v-if="state.isProcessing && !state.messages.some(m => m.isProcessing)"
        class="typing-indicator"
      >
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
    <InputArea ref="inputArea" :is-processing="state.isProcessing" @send="handleSendMessage" />

    <!-- Error Display -->
    <div v-if="state.error" class="error-message">
      <div class="error-content">
        <span class="error-icon"><AppIcon :icon="TriangleAlert" :size="16" /></span>
        <span class="error-text">{{ state.error }}</span>
        <button class="error-close" @click="clearError"><AppIcon :icon="X" :size="14" /></button>
      </div>
    </div>
    <AppPromptDialog
      :visible="saveDialogVisible"
      title="Save Conversation"
      message="Choose a title for this conversation."
      :initial-value="saveDialogTitle"
      placeholder="Conversation title"
      confirm-text="Save"
      @confirm="handleSaveDialogConfirm"
      @cancel="handleSaveDialogCancel"
    />

    <AppConfirmDialog
      :visible="clearDialogVisible"
      title="Clear Conversation"
      message="Are you sure you want to clear this conversation?"
      confirm-text="Clear"
      danger
      @confirm="handleClearDialogConfirm"
      @cancel="handleClearDialogCancel"
    />

    <AppToast :visible="toastVisible" :message="toastMessage" :type="toastType" />
  </div>
</template>

<script setup lang="ts">
  import { ref, reactive, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
  import { invoke } from '@tauri-apps/api/core'
  import { Brain, MessageSquareText, Save, Search, TriangleAlert, Trash2, X } from '@lucide/vue'
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import { setupMarkdownCopyFunction, cleanupMarkdownCopyFunction } from '../utils/markdown'
  import { logger } from '../utils/logger'
  import { formatModelName } from '../utils/formatters'
  import AppIcon from './AppIcon.vue'
  import AppConfirmDialog from './AppConfirmDialog.vue'
  import AppPromptDialog from './AppPromptDialog.vue'
  import AppToast from './AppToast.vue'
  import MessageBubble from './MessageBubble.vue'
  import InputArea from './InputArea.vue'
  import type {
    AIResponse,
    ChatMessage,
    ChatWindowProps,
    Config,
    SavedConversation
  } from '../types'
  import {
    CHAT_MODEL,
    MODEL_CAPABILITIES,
    MODEL_NAMES,
    asModelName,
    isModelName,
    type ModelName
  } from '../types'
  import { useConfirmDialog } from '../composables/useConfirmDialog'
  import { usePromptDialog } from '../composables/usePromptDialog'
  import { useToast } from '../composables/useToast'

  // Props
  const props = withDefaults(defineProps<ChatWindowProps>(), {
    operation: '',
    initialText: '',
    title: 'AI Chat',
    instruction: '',
    conversationId: ''
  })

  // Centralized reactive state
  const state = reactive({
    messages: [] as ChatMessage[],
    isProcessing: false,
    error: null as string | null,
    selectedModel: CHAT_MODEL as ModelName,
    enableThinking: false,
    enableGrounding: false,
    availableModels: [] as ModelName[]
  })

  // Refs
  const messagesContainer = ref<HTMLElement>()
  const inputArea = ref<InstanceType<typeof InputArea>>()

  // Dialog/toast state (via composables)
  const {
    visible: saveDialogVisible,
    initialValue: saveDialogTitle,
    open: openSaveDialog,
    confirm: handleSaveDialogConfirm,
    cancel: handleSaveDialogCancel
  } = usePromptDialog()
  const {
    visible: clearDialogVisible,
    open: openClearDialog,
    confirm: handleClearDialogConfirm,
    cancel: handleClearDialogCancel
  } = useConfirmDialog()
  const {
    visible: toastVisible,
    message: toastMessage,
    type: toastType,
    show: showToast
  } = useToast()

  // Computed Properties
  const windowTitle = computed(() => {
    if (props.title && props.title !== 'AI Chat') return props.title
    if (!props.operation || props.operation === 'Chat') return 'AI Chat'
    return `AI Chat - ${props.operation}`
  })

  const showOperationBadge = computed(() => {
    return Boolean(props.operation && props.operation !== 'Chat')
  })

  const initialTextPreview = computed(() => {
    if (!props.initialText) return ''
    const maxLength = 150
    return props.initialText.length > maxLength
      ? props.initialText.substring(0, maxLength) + '...'
      : props.initialText
  })

  const supportsThinking = computed(() => {
    return MODEL_CAPABILITIES[state.selectedModel]?.thinking ?? false
  })

  const supportsGrounding = computed(() => {
    return MODEL_CAPABILITIES[state.selectedModel]?.grounding ?? false
  })

  // Validation constants
  // Keep in sync with Rust: src-tauri/src/utils/validation.rs `MAX_MESSAGE_LENGTH`
  const MAX_MESSAGE_LENGTH = 10000 // 10KB limit, matching backend
  const MIN_MESSAGE_LENGTH = 1

  // Methods
  const isValidRole = (role: unknown): role is 'user' | 'assistant' => {
    return role === 'user' || role === 'assistant'
  }

  const validateMessage = (message: string): string | null => {
    const trimmed = message.trim()

    if (trimmed.length < MIN_MESSAGE_LENGTH) {
      return 'Message cannot be empty'
    }

    if (trimmed.length > MAX_MESSAGE_LENGTH) {
      return `Message cannot exceed ${MAX_MESSAGE_LENGTH} characters (${(MAX_MESSAGE_LENGTH / 1024).toFixed(1)} KB)`
    }

    return null // Valid message
  }

  const loadAvailableModels = async () => {
    try {
      const models = (await invoke('get_ai_models')) as string[]
      state.availableModels = models.filter(isModelName)
    } catch (err) {
      logger.error('Failed to load models:', err)
      state.availableModels = [...MODEL_NAMES]
    }
  }

  const loadCurrentChatModel = async () => {
    try {
      const config = await invoke<Config>('dm_load_config')
      const active = config.providers?.[config.provider]
      if (active?.chat_model_name) {
        state.selectedModel = asModelName(active.chat_model_name, CHAT_MODEL)
      }
    } catch (err) {
      logger.warn('Failed to load current chat model:', err)
    }
  }

  // Handle send from InputArea component
  const handleSendMessage = async () => {
    if (!inputArea.value) return

    const userMessage = inputArea.value.getCurrentMessage()

    // Validate message
    const validationError = validateMessage(userMessage)
    if (validationError) {
      state.error = validationError
      return
    }

    inputArea.value.clearInput()
    await sendMessage(userMessage)
  }

  // Helper function to prepare chat history for API calls
  const prepareChatHistory = () => {
    return state.messages
      .filter(m => !m.isProcessing)
      .map(m => ({
        role: m.role,
        content: m.content,
        timestamp: m.timestamp
      }))
  }

  const sendMessage = async (userMessage: string) => {
    state.error = null

    // Add user message
    const userMsg: ChatMessage = {
      role: 'user',
      content: userMessage,
      timestamp: new Date().toISOString()
    }
    state.messages.push(userMsg)

    // Add processing AI message
    const aiMsg: ChatMessage = {
      role: 'assistant',
      content: '',
      timestamp: new Date().toISOString(),
      isProcessing: true
    }
    state.messages.push(aiMsg)

    state.isProcessing = true
    await scrollToBottom()

    try {
      // Prepare message history (exclude processing message)
      const chatHistory = prepareChatHistory()

      // Include initial text in first user message if this is the first interaction
      let messageToSend = userMessage
      if (state.messages.length === 2 && props.initialText) {
        messageToSend = `${props.initialText}\n\n${userMessage}`
      }

      // Build instruction with model info
      let instruction = props.instruction || ''
      if (state.selectedModel) {
        instruction += `${instruction ? '\n\n' : ''}Note: You are using ${formatModelName(state.selectedModel)}.`
      }

      // Call backend AI service
      const response = (await invoke('chat_with_ai', {
        message: messageToSend,
        history: chatHistory.slice(0, -1),
        customInstruction: instruction || null,
        selectedModel: state.selectedModel,
        enableThinking: state.enableThinking,
        enableGrounding: state.enableGrounding
      })) as AIResponse

      // Update AI message with response
      const aiIndex = state.messages.findIndex(m => m.isProcessing)
      if (aiIndex !== -1) {
        state.messages[aiIndex] = {
          role: 'assistant',
          content: response.answer,
          timestamp: new Date().toISOString(),
          isProcessing: false,
          thoughts: response.thoughts,
          sources: response.sources,
          searchQueries: response.search_queries
        }
      }
    } catch (err) {
      logger.error('Failed to get AI response:', err)
      state.error = err instanceof Error ? err.message : 'Failed to get AI response'

      // Remove processing message on error
      const processingIndex = state.messages.findIndex(m => m.isProcessing)
      if (processingIndex !== -1) {
        state.messages.splice(processingIndex, 1)
      }
    } finally {
      state.isProcessing = false
      await scrollToBottom()
      void focusInput()
    }
  }

  const regenerateResponse = async (messageIndex: number) => {
    if (messageIndex <= 0 || state.isProcessing) return

    const userMessage = state.messages[messageIndex - 1]
    if (userMessage.role !== 'user') return

    // Remove the AI response and any subsequent messages
    state.messages = state.messages.slice(0, messageIndex)

    // Add processing indicator
    const aiMsg: ChatMessage = {
      role: 'assistant',
      content: '',
      timestamp: new Date().toISOString(),
      isProcessing: true
    }
    state.messages.push(aiMsg)

    state.isProcessing = true
    await scrollToBottom()

    try {
      const chatHistory = prepareChatHistory()

      // Build instruction with model info
      let instruction = props.instruction || ''
      if (state.selectedModel) {
        instruction += `${instruction ? '\n\n' : ''}Note: You are using ${formatModelName(state.selectedModel)}.`
      }

      const response = (await invoke('chat_with_ai', {
        message: userMessage.content,
        history: chatHistory.slice(0, -1),
        customInstruction: instruction || null,
        selectedModel: state.selectedModel,
        enableThinking: state.enableThinking,
        enableGrounding: state.enableGrounding
      })) as AIResponse

      // Update AI message
      const aiIndex = state.messages.findIndex(m => m.isProcessing)
      if (aiIndex !== -1) {
        state.messages[aiIndex] = {
          role: 'assistant',
          content: response.answer,
          timestamp: new Date().toISOString(),
          isProcessing: false,
          thoughts: response.thoughts,
          sources: response.sources,
          searchQueries: response.search_queries
        }
      }
    } catch (err) {
      logger.error('Failed to regenerate response:', err)
      state.error = err instanceof Error ? err.message : 'Failed to regenerate response'

      // Remove processing message
      const processingIndex = state.messages.findIndex(m => m.isProcessing)
      if (processingIndex !== -1) {
        state.messages.splice(processingIndex, 1)
      }
    } finally {
      state.isProcessing = false
      await scrollToBottom()
    }
  }

  const saveConversation = async () => {
    if (state.messages.length === 0) return

    try {
      // Generate a smart default title from first user message
      const firstUserMessage = state.messages.find(m => m.role === 'user')
      const defaultTitle = firstUserMessage
        ? firstUserMessage.content.length > 50
          ? firstUserMessage.content.substring(0, 50) + '...'
          : firstUserMessage.content
        : 'Untitled Conversation'

      // Prompt user for conversation title
      const title = await openSaveDialog({ initialValue: defaultTitle })
      if (!title) return // User cancelled

      // Convert messages to ConversationMessage format
      const conversationMessages = state.messages
        .filter(msg => !msg.isProcessing)
        .map(msg => ({
          role: msg.role,
          content: msg.content,
          timestamp: msg.timestamp,
          thoughts: msg.thoughts,
          sources: msg.sources || [],
          search_queries: msg.searchQueries || []
        }))

      // Save the full conversation
      const conversationId = (await invoke('save_conversation', {
        title: title.trim(),
        operation: props.operation || 'Chat',
        messages: conversationMessages,
        thinkingModeEnabled: state.enableThinking,
        groundingEnabled: state.enableGrounding
      })) as string

      logger.debug('Conversation saved successfully with ID:', conversationId)
      showToast('Conversation saved successfully', 'success')
    } catch (err) {
      logger.error('Failed to save conversation:', err)
      state.error =
        'Failed to save conversation: ' + (err instanceof Error ? err.message : String(err))
      showToast(state.error, 'error')
    }
  }

  const clearConversation = async () => {
    if (state.messages.length === 0) return

    const confirmed = await openClearDialog()
    if (!confirmed) return

    state.messages = []
    state.error = null
    void focusInput()
    showToast('Conversation cleared', 'info')
  }

  const clearError = () => {
    state.error = null
  }

  const scrollToBottom = async () => {
    await nextTick()
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
    }
  }

  const focusInput = async () => {
    await nextTick()
    inputArea.value?.focusInput()
  }

  const closeWindow = () => {
    void getCurrentWindow().close()
  }

  const handleGlobalKeydown = (event: KeyboardEvent) => {
    if (event.ctrlKey || event.metaKey) {
      switch (event.key) {
        case 'l':
        case 'L':
          event.preventDefault()
          void clearConversation()
          break
      }
    } else if (event.key === 'Escape') {
      closeWindow()
    }
  }

  // Load existing conversation
  const loadConversation = async () => {
    if (!props.conversationId) return

    try {
      logger.debug('Loading conversation:', props.conversationId)

      const conversation = (await invoke('load_conversation_messages', {
        conversationId: props.conversationId
      })) as SavedConversation

      // Convert and load messages
      state.messages = conversation.messages.map(msg => {
        // Validate role before using it
        if (!isValidRole(msg.role)) {
          logger.warn(`Invalid role: ${msg.role}, defaulting to assistant`)
          return {
            role: 'assistant' as const,
            content: msg.content,
            timestamp: msg.timestamp,
            isProcessing: false,
            thoughts: msg.thoughts,
            sources: msg.sources,
            searchQueries: msg.search_queries
          }
        }
        return {
          role: msg.role,
          content: msg.content,
          timestamp: msg.timestamp,
          isProcessing: false,
          thoughts: msg.thoughts,
          sources: msg.sources,
          searchQueries: msg.search_queries
        }
      })

      // Restore thinking mode setting
      if (conversation.thinking_mode_enabled !== undefined) {
        state.enableThinking = conversation.thinking_mode_enabled
        logger.debug(`Restored thinking mode: ${conversation.thinking_mode_enabled}`)
      }

      if (conversation.grounding_enabled !== undefined) {
        state.enableGrounding = conversation.grounding_enabled && supportsGrounding.value
        logger.debug(`Restored grounding mode: ${conversation.grounding_enabled}`)
      }

      logger.debug(
        `Loaded conversation "${conversation.title}" with ${conversation.messages.length} messages`
      )

      // Scroll to bottom after messages are loaded
      await nextTick()
      void scrollToBottom()
    } catch (err) {
      logger.error('Failed to load conversation:', err)
      state.error =
        'Failed to load conversation: ' + (err instanceof Error ? err.message : String(err))
    }
  }

  // Watchers
  watch(
    () => state.messages.length,
    () => {
      void scrollToBottom()
    }
  )

  watch(
    () => state.isProcessing,
    newVal => {
      if (!newVal) {
        void focusInput()
      }
    }
  )

  watch(
    () => state.selectedModel,
    model => {
      if (!supportsThinking.value) {
        state.enableThinking = false
      }
      if (!supportsGrounding.value) {
        state.enableGrounding = false
      }
      logger.debug(`Selected chat model changed to: ${model}`)
    }
  )

  // Lifecycle
  onMounted(async () => {
    logger.debug('ChatWindow mounted with props:', {
      operation: props.operation,
      initialText: props.initialText ? `${props.initialText.length} chars` : 'none',
      title: props.title,
      conversationId: props.conversationId
    })

    await Promise.all([loadAvailableModels(), loadCurrentChatModel()])
    void focusInput()

    // Load existing conversation if conversationId is provided
    if (props.conversationId) {
      logger.debug('Loading existing conversation:', props.conversationId)
      await loadConversation()
    } else if (props.initialText && props.operation) {
      logger.debug(`Auto-sending initial text for operation: ${props.operation}`)
      await sendMessage(props.initialText)
    } else {
      logger.debug('No initial text or operation - waiting for user input')
    }

    // Setup global markdown copy function
    setupMarkdownCopyFunction()
  })

  onUnmounted(() => {
    cleanupMarkdownCopyFunction()
  })
</script>

<style scoped>
  .chat-window {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: linear-gradient(135deg, var(--color-bg-surface) 0%, var(--color-bg-app) 100%);
    font-family: var(--font-family-base);
  }

  /* Header Styles */
  .chat-header {
    position: relative;
    display: flex;
    align-items: center;
    padding: var(--space-3) var(--space-5);
    background: var(--color-bg-surface);
    border-bottom: 1px solid var(--color-border-subtle);
    backdrop-filter: blur(10px);
    z-index: 10;
  }

  .header-left {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 0;
  }

  .chat-title {
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-semibold);
    margin: 0;
    color: var(--color-text-primary);
  }

  .operation-badge {
    background: var(--color-accent-soft);
    color: var(--color-accent);
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: var(--font-weight-medium);
    text-transform: capitalize;
  }

  .header-center {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-4);
    padding: 0 var(--space-2);
    max-width: calc(100% - 280px);
  }

  .header-actions {
    display: flex;
    justify-content: flex-end;
    margin-left: auto;
  }

  .model-selector {
    display: flex;
    align-items: center;
    min-width: 220px;
  }

  .model-dropdown {
    min-width: 220px;
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-lg);
    background: var(--input-bg);
    font-size: 13px;
    color: var(--input-text);
    cursor: pointer;
    box-shadow: var(--shadow-sm);
    transition:
      border-color var(--transition-base),
      box-shadow var(--transition-base);
  }

  .model-dropdown:focus {
    outline: none;
    border-color: var(--input-border-focus);
    box-shadow: var(--shadow-focus);
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
    gap: var(--space-3);
    padding: var(--space-2) 10px;
    border: 1px solid var(--color-border-subtle);
    border-radius: 12px;
    background: var(--color-bg-elevated);
    cursor: pointer;
    user-select: none;
    box-shadow: var(--shadow-sm);
  }

  .toggle-label--icon {
    gap: 10px;
  }

  .toggle-label input {
    display: none;
  }

  .thinking-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
  }

  .toggle-slider {
    position: relative;
    width: 36px;
    height: 20px;
    background: var(--color-border);
    border-radius: 10px;
    transition: background var(--transition-slow);
  }

  .toggle-slider::before {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    background: white;
    border-radius: var(--radius-full);
    transition: transform var(--transition-slow);
  }

  .toggle-label input:checked + .toggle-slider {
    background: var(--color-accent);
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
    border-radius: var(--radius-sm);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    transition: all var(--transition-base);
    background: var(--color-bg-elevated);
    color: var(--color-text-secondary);
  }

  .action-btn:hover:not(:disabled) {
    background: var(--color-border);
    transform: scale(1.05);
  }

  .action-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .save-btn {
    color: var(--color-success);
  }
  .clear-btn {
    color: var(--color-danger);
  }
  .close-btn {
    color: var(--color-text-tertiary);
  }

  /* Messages Area */
  .chat-messages {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-5);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .welcome-message {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 200px;
    text-align: center;
    color: var(--color-text-tertiary);
  }

  .welcome-heading {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    color: var(--color-text-primary);
    margin-bottom: var(--space-4);
  }

  .initial-text-preview {
    margin-top: var(--space-4);
    padding: var(--space-4);
    background: var(--color-bg-elevated);
    border-radius: var(--radius-md);
    border-left: 4px solid var(--color-accent);
    text-align: left;
    max-width: 400px;
  }

  .initial-text {
    margin-top: var(--space-2);
    padding: var(--space-2);
    background: var(--color-bg-app);
    border-radius: var(--radius-sm);
    font-family: monospace;
    font-size: 13px;
    color: var(--color-text-primary);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .typing-indicator {
    align-self: flex-start;
    margin: var(--space-2) 0;
  }

  .typing-bubble {
    background: var(--color-bg-elevated);
    padding: var(--space-3) var(--space-4);
    border-radius: 18px;
    border-bottom-left-radius: 6px;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 13px;
    color: var(--color-text-secondary);
  }

  .thinking-dots {
    display: flex;
    gap: 2px;
  }

  .thinking-dots span {
    width: 6px;
    height: 6px;
    background: var(--color-text-muted);
    border-radius: var(--radius-full);
    animation: thinking 1.4s infinite ease-in-out;
  }

  .thinking-dots span:nth-child(1) {
    animation-delay: -0.32s;
  }

  .thinking-dots span:nth-child(2) {
    animation-delay: -0.16s;
  }

  @keyframes thinking {
    0%,
    80%,
    100% {
      transform: scale(0.6);
      opacity: 0.4;
    }
    40% {
      transform: scale(1);
      opacity: 1;
    }
  }

  /* Error Message */
  .error-message {
    position: fixed;
    bottom: var(--space-5);
    left: 50%;
    transform: translateX(-50%);
    z-index: 1000;
  }

  .error-content {
    background: var(--color-danger);
    color: white;
    padding: var(--space-3) var(--space-5);
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    gap: var(--space-3);
    box-shadow: var(--shadow-md);
    max-width: 400px;
  }

  .error-icon,
  .error-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
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

  /* Responsive */
  @media (max-width: 768px) {
    .chat-header {
      flex-direction: column;
      gap: var(--space-3);
      align-items: stretch;
    }

    .header-center {
      position: static;
      left: auto;
      transform: none;
      align-items: stretch;
      justify-content: flex-start;
      flex-wrap: wrap;
      gap: var(--space-2);
      padding: 0;
      max-width: none;
    }

    .header-actions {
      justify-content: flex-end;
      margin-left: 0;
    }

    .model-selector {
      min-width: 0;
      width: 100%;
    }

    .model-dropdown {
      min-width: 0;
      width: 100%;
    }
  }

  @media (max-width: 480px) {
    .chat-messages {
      padding: var(--space-3);
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
