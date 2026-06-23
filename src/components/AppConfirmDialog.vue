<template>
  <div v-if="visible" class="app-dialog-overlay" @click="$emit('cancel')">
    <div class="app-dialog app-dialog--confirm" @click.stop>
      <h3 class="app-dialog-title">{{ title }}</h3>
      <p class="app-dialog-message app-dialog-message--multiline">{{ message }}</p>
      <div class="app-dialog-buttons">
        <button class="app-dialog-button app-dialog-button--cancel" @click="$emit('cancel')">
          {{ cancelText }}
        </button>
        <button
          class="app-dialog-button"
          :class="danger ? 'app-dialog-button--danger' : 'app-dialog-button--confirm'"
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
