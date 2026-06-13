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
    right: var(--space-4);
    bottom: var(--space-4);
    z-index: 2300;
    border-radius: var(--radius-md);
    padding: var(--space-3) var(--space-4);
    color: white;
    font-size: var(--font-size-sm);
    max-width: min(420px, calc(100vw - 32px));
    box-shadow: var(--shadow-md);
  }

  .toast-info {
    background: var(--color-accent);
  }

  .toast-success {
    background: var(--color-success);
  }

  .toast-error {
    background: var(--color-danger);
  }
</style>
