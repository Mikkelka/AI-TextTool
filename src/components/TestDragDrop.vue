<template>
  <div class="test-container">
    <h2>Simple Drag & Drop Test</h2>
    <div class="items-container">
      <div
        v-for="(item, index) in items"
        :key="item.id"
        class="draggable-item"
        :draggable="true"
        @dragstart="handleDragStart(index, $event)"
        @dragend="handleDragEnd"
        @dragover.prevent
        @drop="handleDrop(index, $event)"
        :class="{ dragging: draggedIndex === index }"
      >
        {{ item.name }}
      </div>
    </div>
    <div class="log">
      <h3>Event Log:</h3>
      <pre>{{ eventLog }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const items = ref([
  { id: 1, name: 'Item 1' },
  { id: 2, name: 'Item 2' },
  { id: 3, name: 'Item 3' },
  { id: 4, name: 'Item 4' },
])

const draggedIndex = ref<number | null>(null)
const eventLog = ref('')

const handleDragStart = (index: number, event: DragEvent) => {
  eventLog.value += `Drag start: ${index}\n`
  draggedIndex.value = index
  event.dataTransfer!.effectAllowed = 'move'
}

const handleDragEnd = () => {
  eventLog.value += `Drag end\n`
  draggedIndex.value = null
}

const handleDrop = (dropIndex: number, event: DragEvent) => {
  event.preventDefault()
  eventLog.value += `Drop at: ${dropIndex}\n`
  
  if (draggedIndex.value === null || draggedIndex.value === dropIndex) {
    return
  }
  
  // Reorder items
  const newItems = [...items.value]
  const [draggedItem] = newItems.splice(draggedIndex.value, 1)
  newItems.splice(dropIndex, 0, draggedItem)
  items.value = newItems
  
  draggedIndex.value = null
}
</script>

<style scoped>
.test-container {
  padding: 20px;
  max-width: 600px;
  margin: 0 auto;
}

.items-container {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin: 20px 0;
}

.draggable-item {
  padding: 15px;
  background: #f0f0f0;
  border: 2px solid #ddd;
  border-radius: 8px;
  cursor: move;
  user-select: none;
}

.draggable-item.dragging {
  opacity: 0.5;
  background: #e0e0e0;
}

.draggable-item:hover {
  background: #e8e8e8;
}

.log {
  margin-top: 30px;
  padding: 15px;
  background: #f9f9f9;
  border-radius: 8px;
}

.log pre {
  margin: 10px 0 0 0;
  font-size: 12px;
  max-height: 200px;
  overflow-y: auto;
}
</style>