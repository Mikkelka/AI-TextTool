<template>
  <div v-if="visible" class="dialog-overlay" @click="$emit('cancel')">
    <div class="confirm-dialog" @click.stop>
      <h3 class="dialog-title">{{ title }}</h3>
      <p class="dialog-message">{{ message }}</p>
      <div class="dialog-buttons">
        <button class="dialog-button cancel-button" @click="$emit('cancel')">
          {{ cancelText }}
        </button>
        <button
          class="dialog-button"
          :class="danger ? 'confirm-button-danger' : 'confirm-button'"
          @click="$emit('confirm')"
        >
          {{ confirmText }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
  interface Props {
    visible: boolean
    title: string
    message: string
    confirmText?: string
    cancelText?: string
    danger?: boolean
  }

  withDefaults(defineProps<Props>(), {
    confirmText: 'Confirm',
    cancelText: 'Cancel',
    danger: false
  })

  defineEmits<{
    confirm: []
    cancel: []
  }>()
</script>

<style scoped>
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: var(--color-bg-overlay);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2100;
    padding: var(--space-4);
  }

  .confirm-dialog {
    background: linear-gradient(145deg, var(--color-bg-elevated), var(--color-bg-app));
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-xl);
    width: min(460px, 100%);
    padding: var(--space-5);
    box-shadow:
      var(--shadow-lg),
      inset 0 1px 0 rgba(255, 255, 255, 0.05);
  }

  .dialog-title {
    margin: 0 0 var(--space-2) 0;
    color: var(--color-text-primary);
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-bold);
  }

  .dialog-message {
    margin: 0 0 var(--space-4) 0;
    color: var(--color-text-secondary);
    line-height: 1.45;
    white-space: pre-line;
  }

  .dialog-buttons {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  .dialog-button {
    border: none;
    border-radius: var(--radius-md);
    padding: var(--space-2) var(--space-4);
    font-size: var(--font-size-base);
    cursor: pointer;
    font-weight: var(--font-weight-semibold);
    transition:
      transform var(--transition-fast),
      box-shadow var(--transition-base),
      background-color var(--transition-base);
  }

  .dialog-button:hover {
    transform: translateY(-1px);
  }

  .cancel-button {
    background: var(--color-bg-elevated);
    color: var(--color-text-primary);
  }

  .cancel-button:hover {
    background: var(--color-border);
  }

  .confirm-button {
    background: linear-gradient(135deg, var(--color-accent), var(--color-accent-active));
    color: #fff;
    box-shadow: 0 8px 16px var(--color-accent-soft);
  }

  .confirm-button-danger {
    background: linear-gradient(135deg, var(--color-danger), #dc2626);
    color: #fff;
    box-shadow: 0 8px 16px var(--color-danger-soft);
  }

  .confirm-button:hover {
    box-shadow: 0 10px 20px var(--color-accent-focus-ring);
  }

  .confirm-button-danger:hover {
    box-shadow: 0 10px 20px var(--color-danger-soft);
  }
</style>
