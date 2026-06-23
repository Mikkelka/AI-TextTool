<template>
  <div v-if="visible" class="app-dialog-overlay" @click="$emit('cancel')">
    <div class="app-dialog app-dialog--prompt" @click.stop>
      <h3 class="app-dialog-title">{{ title }}</h3>
      <p class="app-dialog-message">{{ message }}</p>
      <input
        ref="inputRef"
        v-model="draftValue"
        class="form-input"
        type="text"
        :placeholder="placeholder"
        @keydown.enter.prevent="emitConfirm"
      />
      <div class="app-dialog-buttons">
        <button class="app-dialog-button app-dialog-button--cancel" @click="$emit('cancel')">
          {{ cancelText }}
        </button>
        <button class="app-dialog-button app-dialog-button--confirm" @click="emitConfirm">
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
  .form-input {
    width: 100%;
    border: 1px solid var(--input-border);
    border-radius: var(--radius-md);
    padding: var(--space-3);
    font-size: var(--font-size-base);
    margin-bottom: var(--space-4);
    box-sizing: border-box;
    background: var(--input-bg-elevated);
    color: var(--input-text);
    transition:
      border-color var(--transition-base),
      box-shadow var(--transition-base);
  }

  .form-input:focus {
    outline: none;
    border-color: var(--input-border-focus);
    box-shadow: var(--shadow-focus);
  }
</style>
