<template>
  <div v-if="visible" class="dialog-overlay" @click="$emit('cancel')">
    <div class="prompt-dialog" @click.stop>
      <h3 class="dialog-title">{{ title }}</h3>
      <p class="dialog-message">{{ message }}</p>
      <input
        ref="inputRef"
        v-model="draftValue"
        class="dialog-input"
        type="text"
        :placeholder="placeholder"
        @keydown.enter.prevent="emitConfirm"
      />
      <div class="dialog-buttons">
        <button class="dialog-button cancel-button" @click="$emit('cancel')">
          {{ cancelText }}
        </button>
        <button class="dialog-button confirm-button" @click="emitConfirm">
          {{ confirmText }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { nextTick, ref, watch } from 'vue'

  interface Props {
    visible: boolean
    title: string
    message: string
    initialValue?: string
    placeholder?: string
    confirmText?: string
    cancelText?: string
  }

  const props = withDefaults(defineProps<Props>(), {
    initialValue: '',
    placeholder: '',
    confirmText: 'Save',
    cancelText: 'Cancel'
  })

  const emit = defineEmits<{
    confirm: [value: string]
    cancel: []
  }>()

  const draftValue = ref('')
  const inputRef = ref<HTMLInputElement | null>(null)

  watch(
    () => props.visible,
    async visible => {
      if (!visible) return
      draftValue.value = props.initialValue
      await nextTick()
      inputRef.value?.focus()
      inputRef.value?.select()
    }
  )

  const emitConfirm = () => {
    emit('confirm', draftValue.value)
  }
</script>

<style scoped>
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(15, 23, 42, 0.45);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2200;
    padding: 16px;
  }

  .prompt-dialog {
    background: linear-gradient(145deg, rgba(255, 255, 255, 0.96), rgba(243, 246, 251, 0.94));
    border: 1px solid rgba(148, 163, 184, 0.35);
    border-radius: 14px;
    width: min(500px, 100%);
    padding: 22px;
    box-shadow:
      0 14px 38px rgba(15, 23, 42, 0.32),
      inset 0 1px 0 rgba(255, 255, 255, 0.7);
  }

  .dialog-title {
    margin: 0 0 8px 0;
    color: #1e293b;
    font-size: 18px;
    font-weight: 700;
  }

  .dialog-message {
    margin: 0 0 14px 0;
    color: #475569;
    line-height: 1.45;
  }

  .dialog-input {
    width: 100%;
    border: 1px solid #cbd5e1;
    border-radius: 8px;
    padding: 10px 12px;
    font-size: 14px;
    margin-bottom: 14px;
    box-sizing: border-box;
    background: rgba(255, 255, 255, 0.9);
    color: #0f172a;
    box-shadow: inset 0 1px 2px rgba(15, 23, 42, 0.05);
  }

  .dialog-input:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.22);
  }

  .dialog-buttons {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  .dialog-button {
    border: none;
    border-radius: 8px;
    padding: 8px 14px;
    font-size: 14px;
    cursor: pointer;
    font-weight: 600;
    transition:
      transform 0.15s ease,
      box-shadow 0.2s ease,
      background-color 0.2s ease;
  }

  .dialog-button:hover {
    transform: translateY(-1px);
  }

  .cancel-button {
    background: #e2e8f0;
    color: #0f172a;
  }

  .cancel-button:hover {
    box-shadow: 0 6px 12px rgba(15, 23, 42, 0.12);
  }

  .confirm-button {
    background: linear-gradient(135deg, #3b82f6, #2563eb);
    color: #fff;
    box-shadow: 0 8px 16px rgba(37, 99, 235, 0.32);
  }

  .confirm-button:hover {
    box-shadow: 0 10px 20px rgba(37, 99, 235, 0.4);
  }

  @media (prefers-color-scheme: dark) {
    .dialog-overlay {
      background: rgba(2, 6, 23, 0.6);
    }

    .prompt-dialog {
      background: linear-gradient(145deg, rgba(30, 41, 59, 0.96), rgba(15, 23, 42, 0.94));
      border-color: rgba(148, 163, 184, 0.22);
      box-shadow:
        0 14px 38px rgba(2, 6, 23, 0.6),
        inset 0 1px 0 rgba(255, 255, 255, 0.08);
    }

    .dialog-title {
      color: #e2e8f0;
    }

    .dialog-message {
      color: #cbd5e1;
    }

    .dialog-input {
      background: rgba(15, 23, 42, 0.75);
      border-color: #475569;
      color: #e2e8f0;
    }

    .dialog-input:focus {
      border-color: #60a5fa;
      box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.22);
    }

    .cancel-button {
      background: #334155;
      color: #e2e8f0;
    }
  }
</style>
