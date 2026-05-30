<template>
  <div class="chat-input-area">
    <div class="input-container">
      <textarea
        ref="messageInput"
        v-model="currentMessage"
        placeholder="Type your message here... (Press Enter to send, Ctrl+Enter for new line)"
        class="message-input"
        :disabled="isProcessing"
        rows="1"
        @keydown="handleInputKeydown"
      ></textarea>
      <button
        :disabled="!canSend"
        class="send-button"
        title="Send message (Enter)"
        @click="handleSendClick"
      >
        <AppIcon v-if="!isProcessing" :icon="SendHorizontal" :size="18" />
        <AppIcon v-else class="spinner" :icon="LoaderCircle" :size="18" />
      </button>
    </div>

    <!-- Input Status -->
    <div class="input-status">
      <div class="character-count">{{ currentMessage.length }} characters</div>
      <div class="input-hints">
        <span class="hint">Enter to send</span>
        <span class="hint">Ctrl+Enter for new line</span>
        <span class="hint">Escape to close</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { LoaderCircle, SendHorizontal } from '@lucide/vue'
  import { ref, computed, nextTick, onMounted } from 'vue'
  import AppIcon from './AppIcon.vue'

  // Props
  interface Props {
    isProcessing: boolean
  }

  const props = defineProps<Props>()

  // Emits
  const emit = defineEmits<{
    send: []
  }>()

  // State
  const currentMessage = ref('')
  const messageInput = ref<HTMLTextAreaElement>()

  // Computed
  const canSend = computed(() => {
    return currentMessage.value.trim().length > 0 && !props.isProcessing
  })

  // Textarea management helpers
  const resizeTextarea = async () => {
    await nextTick()
    if (messageInput.value) {
      messageInput.value.style.height = 'auto'
      messageInput.value.style.height = Math.min(messageInput.value.scrollHeight, 120) + 'px'
    }
  }

  const resetTextareaHeight = async () => {
    await nextTick()
    if (messageInput.value) {
      messageInput.value.style.height = 'auto'
    }
  }

  // Methods
  const handleInputKeydown = (event: KeyboardEvent) => {
    if (event.key === 'Enter' && !event.shiftKey && !event.ctrlKey) {
      event.preventDefault()
      if (canSend.value && currentMessage.value.trim()) {
        // Emit the send event (parent will handle clearing)
        emit('send')
      }
    } else if (event.key === 'Enter' && event.ctrlKey) {
      // Allow new line
      return
    }

    // Auto-resize textarea
    void resizeTextarea()
  }

  const handleSendClick = () => {
    if (canSend.value && currentMessage.value.trim()) {
      // Emit the send event (parent will handle clearing)
      emit('send')
    }
  }

  const focusInput = async () => {
    await nextTick()
    messageInput.value?.focus()
  }

  const clearInput = () => {
    currentMessage.value = ''
    void resetTextareaHeight()
  }

  const getCurrentMessage = () => {
    return currentMessage.value.trim()
  }

  // Expose methods for parent component
  defineExpose({
    focusInput,
    clearInput,
    getCurrentMessage
  })

  // Auto-focus on mount
  onMounted(() => {
    void focusInput()
  })
</script>

<style scoped>
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
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
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

  /* Responsive */
  @media (max-width: 768px) {
    .input-container {
      gap: 8px;
    }

    .input-hints {
      display: none;
    }
  }

  @media (max-width: 480px) {
    .chat-input-area {
      padding: 12px;
    }
  }
</style>
