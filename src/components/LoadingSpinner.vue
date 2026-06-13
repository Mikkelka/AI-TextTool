<template>
  <span
    class="spinner"
    :class="{ 'spinner--with-margin': margin }"
    :style="spinnerStyle"
    role="status"
    aria-label="Loading"
  ></span>
</template>

<script setup lang="ts">
  import { computed } from 'vue'

  interface Props {
    size?: number
    thickness?: number
    color?: string
    accentColor?: string
    margin?: boolean
  }

  const props = withDefaults(defineProps<Props>(), {
    size: 32,
    thickness: 3,
    color: 'var(--color-border)',
    accentColor: 'var(--color-accent)',
    margin: false
  })

  const spinnerStyle = computed(() => ({
    width: `${props.size}px`,
    height: `${props.size}px`,
    borderWidth: `${props.thickness}px`,
    borderColor: props.color,
    borderTopColor: props.accentColor
  }))
</script>

<style scoped>
  .spinner {
    display: inline-block;
    border-style: solid;
    border-radius: var(--radius-full);
    animation: spinner-rotate 0.8s linear infinite;
  }

  .spinner--with-margin {
    margin-bottom: var(--space-4);
  }

  @keyframes spinner-rotate {
    to {
      transform: rotate(360deg);
    }
  }
</style>
