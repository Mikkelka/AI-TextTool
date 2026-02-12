<template>
  <div v-if="visible" class="toast" :class="`toast-${type}`" role="status" aria-live="polite">
    {{ message }}
  </div>
</template>

<script setup lang="ts">
  interface Props {
    visible: boolean
    message: string
    type?: 'success' | 'error' | 'info'
  }

  withDefaults(defineProps<Props>(), {
    type: 'info'
  })
</script>

<style scoped>
  .toast {
    position: fixed;
    right: 16px;
    bottom: 16px;
    z-index: 2300;
    border-radius: 8px;
    padding: 10px 14px;
    color: #fff;
    font-size: 13px;
    max-width: min(420px, calc(100vw - 32px));
    box-shadow: 0 8px 20px rgba(0, 0, 0, 0.25);
  }

  .toast-info {
    background: #2563eb;
  }

  .toast-success {
    background: #16a34a;
  }

  .toast-error {
    background: #dc2626;
  }
</style>
