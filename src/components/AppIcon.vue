<template>
  <component
    :is="icon"
    :size="size"
    :stroke-width="strokeWidth"
    :aria-hidden="decorative"
    :role="decorative ? 'presentation' : 'img'"
    class="app-icon"
  />
</template>

<script setup lang="ts">
  import type { Component } from 'vue'

  interface Props {
    icon: Component
    size?: number
    strokeWidth?: number
    decorative?: boolean
  }

  withDefaults(defineProps<Props>(), {
    size: 18,
    strokeWidth: 2,
    decorative: true
  })
</script>

<style scoped>
  .app-icon {
    flex-shrink: 0;
    display: block;
  }
</style>
