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
  import { CHAT_MODEL, MODEL_CAPABILITIES, MODEL_NAMES, type ModelName } from '../types'

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
    selectedModel: CHAT_MODEL,
    enableThinking: false,
    enableGrounding: false,
    availableModels: [] as string[]
  })

  // Refs
  const messagesContainer = ref<HTMLElement>()
  const inputArea = ref<InstanceType<typeof InputArea>>()

  // Dialog/toast state
  const saveDialogVisible = ref(false)
  const saveDialogTitle = ref('')
  const clearDialogVisible = ref(false)

  const toastVisible = ref(false)
  const toastMessage = ref('')
  const toastType = ref<'success' | 'error' | 'info'>('info')
  let toastTimer: ReturnType<typeof setTimeout> | null = null

  let saveDialogResolver: ((value: string | null) => void) | null = null
  let clearDialogResolver: ((confirmed: boolean) => void) | null = null

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
    return MODEL_CAPABILITIES[state.selectedModel as ModelName]?.thinking ?? false
  })

  const supportsGrounding = computed(() => {
    return MODEL_CAPABILITIES[state.selectedModel as ModelName]?.grounding ?? false
  })

  // Validation constants
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
      state.availableModels = models
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
        state.selectedModel = active.chat_model_name as ModelName
      }
    } catch (err) {
      logger.warn('Failed to load current chat model:', err)
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

  const showToast = (message: string, type: 'success' | 'error' | 'info' = 'info') => {
    toastMessage.value = message
    toastType.value = type
    toastVisible.value = true

    if (toastTimer) {
      clearTimeout(toastTimer)
    }

    toastTimer = setTimeout(() => {
      toastVisible.value = false
    }, 3200)
  }

  const requestConversationTitle = (defaultTitle: string): Promise<string | null> => {
    saveDialogTitle.value = defaultTitle
    saveDialogVisible.value = true

    return new Promise(resolve => {
      saveDialogResolver = resolve
    })
  }

  const handleSaveDialogConfirm = (value: string) => {
    saveDialogVisible.value = false
    const trimmed = value.trim()
    saveDialogResolver?.(trimmed.length > 0 ? trimmed : null)
    saveDialogResolver = null
  }

  const handleSaveDialogCancel = () => {
    saveDialogVisible.value = false
    saveDialogResolver?.(null)
    saveDialogResolver = null
  }

  const requestClearConfirmation = (): Promise<boolean> => {
    clearDialogVisible.value = true
    return new Promise(resolve => {
      clearDialogResolver = resolve
    })
  }

  const handleClearDialogConfirm = () => {
    clearDialogVisible.value = false
    clearDialogResolver?.(true)
    clearDialogResolver = null
  }

  const handleClearDialogCancel = () => {
    clearDialogVisible.value = false
    clearDialogResolver?.(false)
    clearDialogResolver = null
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
      const title = await requestConversationTitle(defaultTitle)
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

    const confirmed = await requestClearConfirmation()
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
    if (toastTimer) {
      clearTimeout(toastTimer)
      toastTimer = null
    }
    toastVisible.value = false
    if (saveDialogResolver) {
      saveDialogResolver(null)
      saveDialogResolver = null
    }
    if (clearDialogResolver) {
      clearDialogResolver(false)
      clearDialogResolver = null
    }
    cleanupMarkdownCopyFunction()
  })
</script>

<style scoped>
  /* Import all the existing ChatWindow styles but simplified */
  .chat-window {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }

  /* Header Styles */
  .chat-header {
    position: relative;
    display: flex;
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
    min-width: 0;
  }

  .chat-title {
    font-size: 18px;
    font-weight: 600;
    margin: 0;
    color: #333;
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

  .header-center {
    position: absolute;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 0 8px;
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
    padding: 8px 12px;
    border: 1px solid #ddd;
    border-radius: 10px;
    background: white;
    font-size: 13px;
    color: #333;
    cursor: pointer;
    box-shadow: 0 1px 2px rgba(15, 23, 42, 0.04);
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
    gap: 12px;
    padding: 8px 10px;
    border: 1px solid rgba(148, 163, 184, 0.25);
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.82);
    cursor: pointer;
    user-select: none;
    box-shadow: 0 1px 2px rgba(15, 23, 42, 0.04);
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
    color: #64748b;
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

  .save-btn {
    color: #4caf50;
  }
  .clear-btn {
    color: #f44336;
  }
  .zoom-btn {
    color: #2196f3;
  }
  .close-btn {
    color: #666;
  }

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

  .welcome-heading {
    display: inline-flex;
    align-items: center;
    gap: 10px;
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

    .model-dropdown {
      background: #4a5568;
      color: #e2e8f0;
      border-color: #2d3748;
    }

    .toggle-label {
      background: rgba(45, 55, 72, 0.9);
      border-color: rgba(148, 163, 184, 0.2);
    }

    .thinking-icon {
      color: #a0aec0;
    }

    .action-btn {
      background: rgba(45, 55, 72, 0.8);
      color: #cbd5e0;
    }

    .typing-bubble {
      background: rgba(45, 55, 72, 0.9);
      color: #cbd5e0;
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

    .welcome-heading {
      color: #e2e8f0;
    }
  }

  /* Responsive */
  @media (max-width: 768px) {
    .chat-header {
      flex-direction: column;
      gap: 12px;
      align-items: stretch;
    }

    .header-center {
      position: static;
      left: auto;
      transform: none;
      align-items: stretch;
      justify-content: flex-start;
      flex-wrap: wrap;
      gap: 8px;
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
      padding: 12px;
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
