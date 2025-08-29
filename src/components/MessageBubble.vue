<template>
  <div class="message-wrapper">
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
            @click="$emit('regenerate')"
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
</template>

<script setup lang="ts">
import { renderMarkdown } from '../utils/markdown'

// Props
interface Props {
  message: {
    role: 'user' | 'assistant'
    content: string
    timestamp: string
    isProcessing?: boolean
    thoughts?: string
  }
}

defineProps<Props>()

// Emits
defineEmits<{
  regenerate: []
}>()

// Helper functions
const formatTime = (timestamp: string): string => {
  try {
    const date = new Date(timestamp)
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  } catch {
    return new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  }
}

const copyMessage = async (content: string) => {
  try {
    await navigator.clipboard.writeText(content)
  } catch (err) {
    console.error('Failed to copy message:', err)
  }
}
</script>

<style scoped>
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

/* Markdown Content Styles */
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

.processing-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
  color: #666;
  font-size: 13px;
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

/* Thoughts Section */
.thoughts-section {
  margin-bottom: 12px;
}

.thoughts-details {
  background: rgba(173, 216, 230, 0.1);
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
  color: #4682b4;
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

/* Dark mode support */
@media (prefers-color-scheme: dark) {
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
}

/* Responsive */
@media (max-width: 768px) {
  .message-bubble {
    max-width: 95%;
  }
}

@media (max-width: 480px) {
  .message-bubble {
    padding: 12px;
  }
}
</style>