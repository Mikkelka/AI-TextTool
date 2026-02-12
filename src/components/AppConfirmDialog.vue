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
    background: rgba(15, 23, 42, 0.45);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2100;
    padding: 16px;
  }

  .confirm-dialog {
    background: linear-gradient(145deg, rgba(255, 255, 255, 0.96), rgba(243, 246, 251, 0.94));
    border: 1px solid rgba(148, 163, 184, 0.35);
    border-radius: 14px;
    width: min(460px, 100%);
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
    margin: 0 0 16px 0;
    color: #475569;
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

  .confirm-button {
    background: linear-gradient(135deg, #3b82f6, #2563eb);
    color: #fff;
    box-shadow: 0 8px 16px rgba(37, 99, 235, 0.32);
  }

  .confirm-button-danger {
    background: linear-gradient(135deg, #ef4444, #dc2626);
    color: #fff;
    box-shadow: 0 8px 16px rgba(220, 38, 38, 0.32);
  }

  .confirm-button:hover {
    box-shadow: 0 10px 20px rgba(37, 99, 235, 0.4);
  }

  .confirm-button-danger:hover {
    box-shadow: 0 10px 20px rgba(220, 38, 38, 0.4);
  }

  @media (prefers-color-scheme: dark) {
    .dialog-overlay {
      background: rgba(2, 6, 23, 0.6);
    }

    .confirm-dialog {
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

    .cancel-button {
      background: #334155;
      color: #e2e8f0;
    }
  }
</style>
