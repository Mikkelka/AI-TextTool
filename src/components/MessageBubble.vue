<template>
  <div class="message-wrapper">
    <div
      :class="[
        'message-bubble',
        `message-${message.role}`,
        { 'message-processing': message.isProcessing }
      ]"
    >
      <div class="message-header">
        <span class="message-role">
          <AppIcon :icon="message.role === 'user' ? UserRound : Bot" :size="14" />
          {{ message.role === 'user' ? 'You' : 'AI Assistant' }}
        </span>
        <span class="message-time">
          {{ formatTime(message.timestamp) }}
        </span>
        <div class="message-actions">
          <button class="copy-btn" title="Copy message" @click="copyMessage(message.content)">
            <AppIcon :icon="Copy" :size="14" />
          </button>
          <button
            v-if="message.role === 'assistant' && !message.isProcessing"
            class="regenerate-btn"
            title="Regenerate response"
            @click="$emit('regenerate')"
          >
            <AppIcon :icon="RefreshCw" :size="14" />
          </button>
        </div>
      </div>

      <div
        v-if="message.role === 'assistant' && message.thoughts && !message.isProcessing"
        class="thoughts-section"
      >
        <details class="thoughts-details">
          <summary class="thoughts-header">
            <AppIcon :icon="Brain" :size="14" />
            AI's Thinking Process
          </summary>
          <div class="thoughts-content">
            <SanitizedMarkdown
              class="markdown-content thoughts-markdown"
              :markdown="message.thoughts"
            />
          </div>
        </details>
      </div>

      <div class="message-content">
        <SanitizedMarkdown
          v-if="message.role === 'assistant'"
          class="markdown-content"
          :markdown="message.content"
        />
        <div v-else class="user-content">
          {{ message.content }}
        </div>
      </div>

      <div
        v-if="message.role === 'assistant' && message.sources?.length && !message.isProcessing"
        class="sources-section"
      >
        <div class="sources-header">Sources</div>
        <div class="sources-list">
          <a
            v-for="source in message.sources"
            :key="source.uri"
            class="source-chip"
            :href="source.uri"
            @click.prevent="openSource(source.uri)"
          >
            {{ source.title }}
          </a>
        </div>
      </div>

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
  import { Bot, Brain, Copy, RefreshCw, UserRound } from '@lucide/vue'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import AppIcon from './AppIcon.vue'
  import SanitizedMarkdown from './SanitizedMarkdown.vue'
  import { logger } from '../utils/logger'
  import type { ChatMessage } from '../types'

  interface Props {
    message: ChatMessage
  }

  defineProps<Props>()

  defineEmits<{
    regenerate: []
  }>()

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
      logger.error('Failed to copy message:', err)
    }
  }

  const openSource = async (url: string) => {
    try {
      await openUrl(url)
    } catch (err) {
      logger.error('Failed to open source URL:', err)
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
    padding: var(--space-4);
    border-radius: 18px;
    position: relative;
    word-wrap: break-word;
    color: var(--color-text-primary);
  }

  .message-user {
    align-self: flex-end;
    background: linear-gradient(135deg, var(--color-accent-soft) 0%, rgba(37, 99, 235, 0.4) 100%);
    border: 1px solid rgba(59, 130, 246, 0.3);
    border-bottom-right-radius: 6px;
  }

  .message-assistant {
    align-self: flex-start;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-bottom-left-radius: 6px;
    box-shadow: var(--shadow-sm);
  }

  .message-processing {
    opacity: 0.8;
  }

  .message-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-2);
    font-size: var(--font-size-xs);
  }

  .message-role {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-tertiary);
  }

  .message-time {
    color: var(--color-text-muted);
  }

  .message-actions {
    display: flex;
    gap: 4px;
  }

  .copy-btn,
  .regenerate-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    opacity: 0.6;
    transition: all var(--transition-base);
  }

  .copy-btn:hover,
  .regenerate-btn:hover {
    opacity: 1;
    color: var(--color-text-primary);
    background: var(--color-border-subtle);
  }

  .message-content {
    line-height: 1.5;
  }

  .user-content {
    color: var(--color-text-primary);
  }

  .markdown-content {
    color: var(--color-text-primary);
  }

  .markdown-content :deep(h1),
  .markdown-content :deep(h2),
  .markdown-content :deep(h3),
  .markdown-content :deep(h4) {
    margin: var(--space-4) 0 var(--space-2) 0;
    color: var(--color-text-primary);
  }

  .markdown-content :deep(p) {
    margin: var(--space-2) 0;
  }

  .markdown-content :deep(pre) {
    position: relative;
    background: var(--color-bg-app);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: var(--space-3);
    margin: var(--space-3) 0;
    overflow-x: auto;
  }

  .markdown-content :deep(.copy-code-btn) {
    position: absolute;
    top: 8px;
    right: 8px;
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 4px 8px;
    font-size: var(--font-size-xs);
    color: var(--color-text-secondary);
    cursor: pointer;
  }

  .markdown-content :deep(code) {
    font-family: 'Monaco', 'Consolas', monospace;
    font-size: 13px;
  }

  .markdown-content :deep(.inline-code) {
    background: var(--color-bg-elevated);
    padding: 2px 4px;
    border-radius: 3px;
    font-size: 13px;
    color: var(--color-text-primary);
  }

  .markdown-content :deep(ul),
  .markdown-content :deep(ol) {
    margin: var(--space-3) 0;
    padding-left: var(--space-6);
  }

  .markdown-content :deep(li) {
    margin: 6px 0;
  }

  .markdown-content :deep(blockquote) {
    margin: var(--space-3) 0;
    padding: var(--space-3) var(--space-4);
    border-left: 4px solid var(--color-accent);
    background: var(--color-accent-soft);
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    color: var(--color-text-secondary);
    font-style: italic;
  }

  .markdown-content :deep(a) {
    color: var(--color-accent);
    text-decoration: none;
  }

  .markdown-content :deep(a:hover) {
    text-decoration: underline;
  }

  .markdown-content :deep(table) {
    border-collapse: collapse;
    width: 100%;
    margin: var(--space-3) 0;
    background: var(--color-bg-app);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .markdown-content :deep(thead) {
    background: var(--color-bg-elevated);
  }

  .markdown-content :deep(th),
  .markdown-content :deep(td) {
    padding: 10px var(--space-3);
    text-align: left;
    border-bottom: 1px solid var(--color-border);
    color: var(--color-text-primary);
  }

  .markdown-content :deep(th) {
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-primary);
  }

  .markdown-content :deep(tbody tr:hover) {
    background: var(--color-border-subtle);
  }

  .markdown-content :deep(hr) {
    margin: var(--space-5) 0;
    border: none;
    height: 1px;
    background: var(--color-border);
  }

  .processing-indicator {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-2);
    color: var(--color-text-tertiary);
    font-size: 13px;
  }

  .sources-section {
    margin-top: 14px;
    padding-top: var(--space-3);
    border-top: 1px solid var(--color-border-subtle);
  }

  .sources-header {
    margin-bottom: var(--space-2);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-bold);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .sources-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .source-chip {
    display: inline-flex;
    align-items: center;
    max-width: 100%;
    padding: 6px 10px;
    border-radius: var(--radius-full);
    background: var(--color-accent-soft);
    color: var(--color-accent);
    font-size: var(--font-size-xs);
    text-decoration: none;
    border: 1px solid rgba(59, 130, 246, 0.3);
    transition: all var(--transition-base);
  }

  .source-chip:hover {
    background: rgba(59, 130, 246, 0.22);
    text-decoration: none;
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

  .thoughts-section {
    margin-bottom: var(--space-3);
  }

  .thoughts-details {
    background: var(--color-accent-soft);
    border: 1px solid rgba(59, 130, 246, 0.3);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .thoughts-header {
    background: rgba(59, 130, 246, 0.18);
    padding: var(--space-2) var(--space-3);
    cursor: pointer;
    font-size: 13px;
    font-weight: var(--font-weight-medium);
    color: var(--color-accent);
    display: flex;
    align-items: center;
    gap: 6px;
    user-select: none;
    border: none;
    outline: none;
  }

  .thoughts-header:hover {
    background: rgba(59, 130, 246, 0.28);
  }

  .thoughts-content {
    padding: var(--space-3);
    font-size: 13px;
    line-height: 1.5;
  }

  .thoughts-markdown {
    color: var(--color-text-primary);
    font-style: italic;
  }

  .thoughts-markdown :deep(p) {
    margin: 6px 0;
  }

  .thoughts-markdown :deep(code) {
    background: rgba(59, 130, 246, 0.2);
    color: var(--color-accent);
  }

  .thoughts-markdown :deep(pre) {
    background: rgba(59, 130, 246, 0.15);
    border-color: rgba(59, 130, 246, 0.3);
  }
</style>
