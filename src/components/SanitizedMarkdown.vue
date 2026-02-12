<template>
  <!-- Content comes from renderMarkdown(), which sanitizes with DOMPurify before binding. -->
  <!-- eslint-disable-next-line vue/no-v-html -->
  <div v-html="sanitizedHtml"></div>
</template>

<script setup lang="ts">
  import { computed } from 'vue'
  import { renderMarkdown } from '../utils/markdown'

  interface Props {
    markdown: string
  }

  const props = defineProps<Props>()

  const sanitizedHtml = computed(() => renderMarkdown(props.markdown))
</script>
