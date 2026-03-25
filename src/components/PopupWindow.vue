<template>
  <div class="popup-container" tabindex="0" @keydown="handleKeydown">
    <!-- Loading State -->
    <div v-if="isLoading" class="loading-container">
      <div class="loading-spinner"></div>
      <p class="loading-text">Loading operations...</p>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="error-container">
      <div class="error-icon"><AppIcon :icon="TriangleAlert" :size="28" /></div>
      <p class="error-message">{{ error }}</p>
      <button class="retry-button" @click="loadOperations">Retry</button>
    </div>

    <!-- Main Content -->
    <div v-else class="popup-content">
      <!-- Minimal Header -->
      <div class="popup-header" data-tauri-drag-region>
        <span class="popup-title" data-tauri-drag-region><AppIcon :icon="Sparkles" :size="14" />AI Operations</span>
        <button
          class="popup-close-btn"
          title="Close (ESC)"
          data-tauri-drag-region="false"
          @click="closeWindow"
        >
          <AppIcon :icon="X" :size="12" />
        </button>
      </div>

      <!-- Operations Grid -->
      <div class="operations-grid">
        <button
          v-for="([key, operation], index) in operations"
          :key="key"
          :ref="el => (buttonRefs[index] = el as HTMLElement)"
          :class="[
            'operation-button',
            {
              'operation-button--chat': operation.open_in_window,
              'operation-button--direct': !operation.open_in_window,
              'operation-button--selected': selectedIndex === index,
              'operation-button--processing': processingOperation === key
            }
          ]"
          :disabled="processingOperation !== null"
          :title="getOperationTooltip(operation)"
          @click="handleOperationClick(key, operation)"
          @mouseenter="selectedIndex = index"
        >
          <div class="operation-button-content">
            <div class="operation-label">
              {{ processingOperation === key ? 'Processing...' : key }}
            </div>
          </div>
        </button>
      </div>

      <!-- Processing Indicator -->
      <div v-if="processingOperation" class="processing-indicator">
        <div class="processing-spinner"></div>
        <p>Processing with AI...</p>
      </div>
    </div>

    <!-- Fade-in animation -->
    <div class="fade-overlay" :class="{ 'fade-overlay--hidden': !showFadeIn }"></div>
  </div>
</template>

<script setup lang="ts">
  import { ref, onMounted, onUnmounted, nextTick } from 'vue'
  import { invoke } from '@tauri-apps/api/core'
  import { Sparkles, TriangleAlert, X } from '@lucide/vue'
  import AppIcon from './AppIcon.vue'
  import { logger } from '../utils/logger'
  import type { Operation, PopupWindowProps } from '../types'

  // Props
  const props = withDefaults(defineProps<PopupWindowProps>(), {
    selectedText: ''
  })

  // Emits
  interface Emits {
    (e: 'close'): void
    (e: 'operation-selected', operation: string, details: Operation): void
  }

  const emit = defineEmits<Emits>()

  // Layout constants
  const GRID_COLUMNS = 2 // Number of columns in the operations grid

  // Reactive state
  const operations = ref<[string, Operation][]>([])
  const isLoading = ref(true)
  const error = ref<string | null>(null)
  const processingOperation = ref<string | null>(null)
  const selectedIndex = ref(0)
  const buttonRefs = ref<(HTMLElement | null)[]>([])
  const showFadeIn = ref(true)

  // Get text from props or window.clipboardText injection
  const clipboardText = ref(props.selectedText || window.clipboardText || '')

  // Methods
  const loadOperations = async () => {
    try {
      isLoading.value = true
      error.value = null

      const result = (await invoke('dm_load_operations_sorted')) as [string, Operation][]
      operations.value = result

      // Reset selected index if operations changed
      if (selectedIndex.value >= result.length) {
        selectedIndex.value = 0
      }
    } catch (err) {
      logger.error('Failed to load operations:', err)
      error.value = err instanceof Error ? err.message : 'Failed to load operations'
    } finally {
      isLoading.value = false
      // Hide fade-in overlay after loading
      setTimeout(() => {
        showFadeIn.value = false
      }, 300)
    }
  }

  const handleOperationClick = async (operationKey: string, operation: Operation) => {
    logger.debug(`Operation clicked: ${operationKey}, open_in_window: ${operation.open_in_window}`)

    if (!clipboardText.value?.trim()) {
      logger.warn('No text selected for processing')
      error.value = 'No text selected for processing'
      return
    }

    logger.debug(
      `Processing operation: ${operationKey} with text length: ${clipboardText.value.length}`
    )

    try {
      processingOperation.value = operationKey
      emit('operation-selected', operationKey, operation)

      if (operation.open_in_window) {
        logger.debug(`Opening chat window for operation: ${operationKey}`)
        // Open chat window for chat operations
        await openChatWindow(operationKey, operation)
      } else {
        logger.debug(`Processing text directly for operation: ${operationKey}`)
        // Process directly for non-chat operations
        await processTextDirectly(operationKey, operation)
      }
    } catch (err) {
      logger.error('Operation failed:', err)
      error.value = err instanceof Error ? err.message : 'Operation failed'
    } finally {
      processingOperation.value = null
    }
  }

  const processTextDirectly = async (operationKey: string, _operation: Operation) => {
    try {
      const result = (await invoke('process_text_with_ai', {
        text: clipboardText.value,
        operation: operationKey
      })) as string

      logger.debug('Text processed successfully:', result)

      // Save to chat history
      await invoke('save_chat_entry', {
        originalText: clipboardText.value,
        aiOption: operationKey,
        processedText: result
      })

      // Close window first
      void closeWindow()

      // Small delay then simulate paste to replace original text
      setTimeout(async () => {
        try {
          await invoke('simulate_paste')
          logger.debug('Auto-paste completed')
        } catch (pasteError) {
          logger.error('Auto-paste failed:', pasteError)
        }
      }, 200)
    } catch (err) {
      throw new Error(`Failed to process text: ${err}`)
    }
  }

  const openChatWindow = async (operationKey: string, operation: Operation) => {
    try {
      logger.debug(`Opening chat window for operation: ${operationKey}`)

      // Send raw text - let the backend handle operation prefix
      const textToSend = clipboardText.value
      logger.debug(
        `Text to send (length: ${textToSend.length}):`,
        textToSend.substring(0, 100) + '...'
      )

      // Use backend command instead of frontend WebviewWindow
      await invoke('open_chat_window', {
        operation: operationKey,
        text: textToSend,
        instruction: operation.instruction
      })

      logger.debug('Chat window opened successfully via backend command')

      // Close popup after opening chat
      void closeWindow()
    } catch (err) {
      logger.error('Failed to open chat window:', err)
      throw new Error(`Failed to open chat window: ${err}`)
    }
  }

  const handleKeydown = async (event: KeyboardEvent) => {
    const operationCount = operations.value.length

    switch (event.key) {
      case 'Escape':
        event.preventDefault()
        void closeWindow()
        break

      case 'ArrowUp':
        event.preventDefault()
        selectedIndex.value = selectedIndex.value > 0 ? selectedIndex.value - 1 : operationCount - 1
        void scrollToSelected()
        break

      case 'ArrowDown':
        event.preventDefault()
        selectedIndex.value = selectedIndex.value < operationCount - 1 ? selectedIndex.value + 1 : 0
        void scrollToSelected()
        break

      case 'ArrowLeft': {
        event.preventDefault()
        selectedIndex.value = Math.max(0, selectedIndex.value - GRID_COLUMNS)
        void scrollToSelected()
        break
      }

      case 'ArrowRight': {
        event.preventDefault()
        selectedIndex.value = Math.min(operationCount - 1, selectedIndex.value + GRID_COLUMNS)
        void scrollToSelected()
        break
      }

      case 'Enter':
        event.preventDefault()
        if (selectedIndex.value < operationCount && processingOperation.value === null) {
          const [key, operation] = operations.value[selectedIndex.value]
          await handleOperationClick(key, operation)
        }
        break
    }
  }

  const scrollToSelected = async () => {
    await nextTick()
    const button = buttonRefs.value[selectedIndex.value] as HTMLElement | null
    if (button) {
      button.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
    }
  }

  const getOperationTooltip = (operation: Operation): string => {
    const type = operation.open_in_window ? 'Opens chat window' : 'Direct processing'
    const instruction = operation.instruction
      ? `\n\nInstruction: ${operation.instruction.substring(0, 100)}...`
      : ''
    return `${type}${instruction}`
  }

  const closeWindow = async () => {
    try {
      logger.debug('Closing popup window...')
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      const currentWindow = getCurrentWindow()
      await currentWindow.close()
    } catch (error) {
      logger.error('Failed to close popup window:', error)
      // Fallback: emit close event
      emit('close')
    }
  }

  // Lifecycle
  onMounted(() => {
    void loadOperations()

    // Set up keyboard event listener for arrow key navigation
    document.addEventListener('keydown', handleKeydown)
  })

  onUnmounted(() => {
    // Cleanup keyboard event listener
    document.removeEventListener('keydown', handleKeydown)

    // Clear sensitive data
    clipboardText.value = ''
    delete window.clipboardText
  })
</script>

<style scoped>
  .popup-container {
    position: relative;
    width: 100%;
    height: 100vh;
    background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
    margin: 0;
    padding: 0;
    box-sizing: border-box;
    outline: none;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .loading-container,
  .error-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    text-align: center;
  }

  .loading-spinner,
  .processing-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid #f3f3f3;
    border-top: 3px solid #2196f3;
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
    margin-bottom: 8px;
  }

  .error-message {
    margin: 8px 0;
    font-size: 14px;
  }

  .retry-button {
    background: #2196f3;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    transition: background-color 0.2s;
  }

  .retry-button:hover {
    background: #1976d2;
  }

  .popup-content {
    max-height: 100vh;
    overflow-y: auto;
  }

  .popup-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: transparent;
    color: white;
    font-size: 12px;
    min-height: 28px;
  }

  .popup-title {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-weight: 500;
    user-select: none;
  }

  .popup-close-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.2);
    color: white;
    border: none;
    border-radius: 3px;
    width: 16px;
    height: 16px;
    font-size: 10px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
    padding: 0;
  }

  .popup-close-btn:hover {
    background: rgba(255, 255, 255, 0.3);
  }

  .operations-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 6px;
    max-width: 300px;
    max-height: 400px;
    overflow-y: auto;
    padding: 8px;
  }

  .operation-button {
    background: white;
    border: 2px solid transparent;
    border-radius: 6px;
    padding: 12px 6px;
    cursor: pointer;
    transition: all 0.2s ease;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.1);
    min-height: 55px;
    max-height: 55px;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
    width: 100%;
  }

  .operation-button:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  }

  .operation-button--selected {
    border-color: #2196f3;
    transform: translateY(-2px);
    box-shadow: 0 4px 16px rgba(33, 150, 243, 0.3);
  }

  .operation-button--chat {
    background: linear-gradient(135deg, #e3f2fd 0%, #bbdefb 100%);
    border-color: rgba(33, 150, 243, 0.3);
  }

  .operation-button--chat:hover:not(:disabled) {
    background: linear-gradient(135deg, #bbdefb 0%, #90caf9 100%);
    border-color: #2196f3;
  }

  .operation-button--direct {
    background: linear-gradient(135deg, #f3e5f5 0%, #e1bee7 100%);
    border-color: rgba(156, 39, 176, 0.3);
  }

  .operation-button--direct:hover:not(:disabled) {
    background: linear-gradient(135deg, #e1bee7 0%, #ce93d8 100%);
    border-color: #9c27b0;
  }

  .operation-button--processing {
    opacity: 0.7;
    pointer-events: none;
  }

  .operation-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }

  .operation-button-content {
    text-align: center;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
  }

  .operation-label {
    font-size: 13px;
    font-weight: 600;
    color: #333;
    line-height: 1.1;
    text-align: center;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  .processing-indicator {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    background: rgba(33, 150, 243, 0.95);
    color: white;
    padding: 12px 20px;
    border-radius: 24px;
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 14px;
    font-weight: 500;
    backdrop-filter: blur(10px);
    z-index: 20;
  }

  .processing-indicator .processing-spinner {
    width: 16px;
    height: 16px;
    border-width: 2px;
    border-color: rgba(255, 255, 255, 0.3);
    border-top-color: white;
    margin: 0;
  }

  .fade-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(255, 255, 255, 0.8);
    pointer-events: none;
    transition: opacity 0.3s ease;
    z-index: 5;
  }

  .fade-overlay--hidden {
    opacity: 0;
  }

  @keyframes spin {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }

  /* Dark mode support */
  @media (prefers-color-scheme: dark) {
    .popup-container {
      background: linear-gradient(135deg, #2d3748 0%, #1a202c 100%);
    }

    .loading-text {
      color: #a0aec0;
    }

    .operation-button {
      background: rgba(45, 55, 72, 0.8);
      color: #e2e8f0;
    }

    .operation-button--chat {
      background: linear-gradient(135deg, rgba(59, 130, 246, 0.2) 0%, rgba(37, 99, 235, 0.3) 100%);
      border-color: rgba(59, 130, 246, 0.4);
    }

    .operation-button--direct {
      background: linear-gradient(135deg, rgba(139, 92, 246, 0.2) 0%, rgba(124, 58, 237, 0.3) 100%);
      border-color: rgba(139, 92, 246, 0.4);
    }

    .operation-label {
      color: #e2e8f0;
    }

    .fade-overlay {
      background: rgba(45, 55, 72, 0.8);
    }
  }

  /* Responsive adjustments */
  @media (max-width: 350px) {
    .operations-grid {
      grid-template-columns: repeat(2, 1fr);
      gap: 6px;
    }

    .operation-button {
      min-height: 40px;
      max-height: 40px;
      padding: 6px 4px;
    }

    .operation-label {
      font-size: 12px;
      white-space: normal;
      line-height: 1.1;
    }
  }
</style>
