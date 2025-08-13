<template>
  <div class="edit-window-container" @keydown="handleKeydown" tabindex="0">
    <!-- Loading State -->
    <div v-if="isLoading" class="loading-container">
      <div class="loading-spinner"></div>
      <p class="loading-text">Loading operations...</p>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="error-container">
      <div class="error-icon">⚠️</div>
      <p class="error-message">{{ error }}</p>
      <button @click="loadOperations" class="retry-button">Retry</button>
    </div>

    <!-- Main Content -->
    <div v-else class="edit-window-content">
      <!-- Header -->
      <div class="header">
        <h1 class="title">Edit AI Text Operations</h1>
        <p class="instructions">Click edit/delete icons to modify operations</p>
        <div class="header-buttons">
          <button @click="addNewOperation" class="header-button add-button">
            + Add New
          </button>
          <button @click="resetToDefaults" class="header-button reset-button">
            Reset to Defaults
          </button>
        </div>
      </div>

      <!-- Operations Grid -->
      <div class="operations-grid-container">
        <p class="drag-instructions">Use arrow buttons to rearrange order • Click edit/delete icons to modify operations</p>
        <div class="operations-grid">
          <div
            v-for="([key, operation], index) in operationsArray"
            :key="key"
            class="operation-item"
          >
            <div 
              class="operation-button" 
              :class="{ 'chat-operation': operation.open_in_window }"
            >
              <div class="reorder-controls">
                <button
                  @click="moveOperation(index, -1)"
                  class="arrow-button"
                  :disabled="index === 0"
                  title="Move up"
                >
                  ▲
                </button>
                <button
                  @click="moveOperation(index, 1)"
                  class="arrow-button"
                  :disabled="index === operationsArray.length - 1"
                  title="Move down"
                >
                  ▼
                </button>
              </div>
              <div class="operation-content">
                <div class="operation-name">{{ key }}</div>
                <div class="operation-type">
                  {{ operation.open_in_window ? 'Chat Window' : 'Direct Replace' }}
                </div>
              </div>
              
              <!-- Edit/Delete Icons Overlay -->
              <div class="operation-icons">
                <button
                  @click="editOperation(key)"
                  class="icon-button edit-icon"
                  title="Edit operation"
                >
                  ✏️
                </button>
                <button
                  @click="deleteOperation(key)"
                  class="icon-button delete-icon"
                  title="Delete operation"
                >
                  ❌
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

    </div>

    <!-- Confirmation Dialog -->
    <div v-if="showConfirmDialog" class="dialog-overlay" @click="cancelConfirm">
      <div class="confirm-dialog" @click.stop>
        <h3 class="dialog-title">{{ confirmTitle }}</h3>
        <p class="dialog-message">{{ confirmMessage }}</p>
        <div class="dialog-buttons">
          <button @click="cancelConfirm" class="dialog-button cancel-button">Cancel</button>
          <button @click="confirmAction" class="dialog-button confirm-button">{{ confirmButtonText }}</button>
        </div>
      </div>
    </div>

    <!-- Edit Dialog -->
    <div v-if="showEditDialog" class="dialog-overlay" @click="cancelEdit">
      <div class="edit-dialog" @click.stop>
        <h3 class="dialog-title">{{ editingOperation ? 'Edit Operation' : 'Add New Operation' }}</h3>
        
        <div class="form-group">
          <label class="form-label">Operation Name:</label>
          <input
            v-model="editForm.name"
            type="text"
            class="form-input"
            placeholder="Enter operation name"
            :disabled="isEditingExisting"
          />
        </div>

        <div class="form-group">
          <label class="form-label">Prefix Text:</label>
          <textarea
            v-model="editForm.prefix"
            class="form-textarea"
            placeholder="Text to prepend before selected text (optional)"
            rows="2"
          ></textarea>
        </div>

        <div class="form-group">
          <label class="form-label">AI Instruction:</label>
          <textarea
            v-model="editForm.instruction"
            class="form-textarea"
            placeholder="Instructions for the AI on how to process the text"
            rows="4"
          ></textarea>
        </div>


        <div class="form-group">
          <label class="checkbox-label">
            <input
              v-model="editForm.open_in_window"
              type="checkbox"
              class="form-checkbox"
            />
            Open in chat window (instead of direct text replacement)
          </label>
        </div>

        <div class="dialog-buttons">
          <button @click="cancelEdit" class="dialog-button cancel-button">Cancel</button>
          <button @click="saveOperation" class="dialog-button save-button" :disabled="!editForm.name.trim()">
            {{ editingOperation ? 'Update' : 'Add' }} Operation
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'

// Props
interface Props {
  // No props needed for now
}

withDefaults(defineProps<Props>(), {})

// Emits
interface Emits {
  (e: 'close'): void
}

defineEmits<Emits>()

// Types
interface Operation {
  prefix: string
  instruction: string
  icon?: string
  open_in_window: boolean
}

interface EditForm {
  name: string
  prefix: string
  instruction: string
  icon: string
  open_in_window: boolean
}

// Reactive state
const operations = ref<Record<string, Operation>>({})
const isLoading = ref(true)
const error = ref<string | null>(null)

// Operations array for ordered display
const operationsArray = ref<Array<[string, Operation]>>([])


// Dialog states
const showConfirmDialog = ref(false)
const showEditDialog = ref(false)
const confirmTitle = ref('')
const confirmMessage = ref('')
const confirmButtonText = ref('Confirm')
const confirmCallback = ref<(() => void) | null>(null)

// Edit form state
const editForm = ref<EditForm>({
  name: '',
  prefix: '',
  instruction: '',
  icon: '',
  open_in_window: false
})
const editingOperation = ref<string | null>(null)
const isEditingExisting = ref(false)

// Methods
const loadOperations = async () => {
  try {
    isLoading.value = true
    error.value = null
    
    // Load operations as sorted array
    const sortedResult = await invoke('dm_load_operations_sorted') as Array<[string, Operation]>
    operationsArray.value = sortedResult
    
    // Also populate operations object for compatibility
    operations.value = {}
    sortedResult.forEach(([key, operation]) => {
      operations.value[key] = operation
    })
    
    console.log('Loaded operations in order:', sortedResult)
    
  } catch (err) {
    console.error('Failed to load operations:', err)
    error.value = err instanceof Error ? err.message : 'Failed to load operations'
  } finally {
    isLoading.value = false
  }
}

const editOperation = (operationKey: string) => {
  const operation = operations.value[operationKey]
  if (!operation) return
  
  editForm.value = {
    name: operationKey,
    prefix: operation.prefix || '',
    instruction: operation.instruction || '',
    icon: operation.icon || '',
    open_in_window: operation.open_in_window || false
  }
  
  editingOperation.value = operationKey
  isEditingExisting.value = true
  showEditDialog.value = true
}

const addNewOperation = () => {
  editForm.value = {
    name: '',
    prefix: '',
    instruction: '',
    icon: '',
    open_in_window: false
  }
  
  editingOperation.value = null
  isEditingExisting.value = false
  showEditDialog.value = true
}

const deleteOperation = (operationKey: string) => {
  confirmTitle.value = 'Confirm Delete'
  confirmMessage.value = `Are you sure you want to delete the '${operationKey}' operation?`
  confirmButtonText.value = 'Delete'
  confirmCallback.value = () => performDelete(operationKey)
  showConfirmDialog.value = true
}

const performDelete = async (operationKey: string) => {
  try {
    console.log('Attempting to delete operation:', operationKey)
    const success = await invoke('dm_remove_operation', { name: operationKey }) as boolean
    console.log('Delete result:', success)
    
    // Remove from operationsArray to maintain order
    const indexToRemove = operationsArray.value.findIndex(([key]) => key === operationKey)
    if (indexToRemove !== -1) {
      operationsArray.value.splice(indexToRemove, 1)
    }
    
    // Also remove from operations object
    delete operations.value[operationKey]
    
    if (success) {
      console.log('Operation deleted successfully:', operationKey)
    } else {
      console.log('Backend reported failure, but updating UI anyway')
      // Only reload if backend failed - this preserves order
      await loadOperations()
    }
    
    
  } catch (err) {
    console.error('Failed to delete operation:', err)
    error.value = err instanceof Error ? err.message : 'Failed to delete operation'
    // Reload operations to ensure UI is in sync with backend
    await loadOperations()
  }
}

const resetToDefaults = () => {
  confirmTitle.value = 'Reset to Defaults'
  confirmMessage.value = 'Are you sure you want to reset all operations to their default configuration? This will remove any custom operations you have added.'
  confirmButtonText.value = 'Reset'
  confirmCallback.value = () => performReset()
  showConfirmDialog.value = true
}

const performReset = async () => {
  try {
    await invoke('dm_reset_operations')
    
    // Reload operations to show the defaults
    await loadOperations()
    
    console.log('Operations reset to defaults successfully')
    showMessage('Reset Complete', 'All operations have been reset to their default configuration!')
  } catch (err) {
    console.error('Failed to reset operations:', err)
    error.value = err instanceof Error ? err.message : 'Failed to reset operations'
  }
}

const saveOperation = async () => {
  if (!editForm.value.name.trim()) return
  
  try {
    const operation: Operation = {
      prefix: editForm.value.prefix,
      instruction: editForm.value.instruction || 'You are a helpful writing assistant. Follow the user\'s instructions precisely and provide clear, accurate assistance with their text.',
      icon: undefined,
      open_in_window: editForm.value.open_in_window
    }

    await invoke('dm_update_operation', { 
      name: editForm.value.name.trim(), 
      operation: operation 
    })

    // Update local state
    operations.value[editForm.value.name.trim()] = operation

    // If we were editing and the name changed, remove the old one
    if (editingOperation.value && editingOperation.value !== editForm.value.name.trim()) {
      delete operations.value[editingOperation.value]
    }

    // Update operations array manually
    const existingIndex = operationsArray.value.findIndex(([key]) => key === editForm.value.name.trim())
    if (existingIndex !== -1) {
      // Update existing operation
      operationsArray.value[existingIndex] = [editForm.value.name.trim(), operation]
    } else {
      // Add new operation at the end
      operationsArray.value.push([editForm.value.name.trim(), operation])
    }

    console.log('Operation saved:', editForm.value.name, operation)
    showEditDialog.value = false
    
    console.log(`Operation ${editingOperation.value ? 'updated' : 'added'}:`, editForm.value.name)
    
  } catch (err) {
    console.error('Failed to save operation:', err)
    error.value = err instanceof Error ? err.message : 'Failed to save operation'
  }
}

const cancelEdit = () => {
  showEditDialog.value = false
  editingOperation.value = null
  isEditingExisting.value = false
}

const confirmAction = () => {
  if (confirmCallback.value) {
    confirmCallback.value()
  }
  cancelConfirm()
}

const cancelConfirm = () => {
  showConfirmDialog.value = false
  confirmCallback.value = null
}

const closeWindow = async () => {
  try {
    await getCurrentWindow().close()
  } catch (error) {
    console.error('Error closing window:', error)
  }
}

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    if (showEditDialog.value) {
      cancelEdit()
    } else if (showConfirmDialog.value) {
      cancelConfirm()
    } else {
      closeWindow()
    }
  }
}

const showMessage = (title: string, message: string) => {
  // Simple alert for now - could be replaced with a toast notification
  alert(`${title}\n\n${message}`)
}


// Move operation up or down
const moveOperation = async (index: number, direction: number) => {
  const newIndex = index + direction
  
  // Check bounds
  if (newIndex < 0 || newIndex >= operationsArray.value.length) {
    return
  }
  
  try {
    // Reorder the operations array
    const newArray = [...operationsArray.value]
    const temp = newArray[index]
    newArray[index] = newArray[newIndex]
    newArray[newIndex] = temp
    
    // Update order fields to match new positions
    const newOperations: Record<string, Operation> = {}
    newArray.forEach(([key, operation], idx) => {
      const updatedOperation = { ...operation, order: idx + 1 }
      newOperations[key] = updatedOperation
    })
    
    // Update local state immediately for responsive UI
    operations.value = newOperations
    operationsArray.value = newArray
    
    // Save the new order to backend
    await invoke('dm_save_operations', { operations: newOperations })
    
    console.log('Operation moved successfully')
    
  } catch (err) {
    console.error('Failed to move operation:', err)
    error.value = err instanceof Error ? err.message : 'Failed to move operation'
    
    // Reload operations on error to reset to server state
    await loadOperations()
  }
}

// Lifecycle
onMounted(() => {
  loadOperations()
})

onUnmounted(() => {
  // Cleanup if needed
})
</script>

<style scoped>
.edit-window-container {
  position: relative;
  width: 100%;
  height: 100vh;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
  padding: 20px;
  box-sizing: border-box;
  outline: none;
  overflow: auto;
}

.loading-container,
.error-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 50vh;
  text-align: center;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #f3f3f3;
  border-top: 3px solid #2196F3;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 16px;
}

.loading-text {
  color: #666;
  font-size: 14px;
  margin: 0;
}

.error-container {
  color: #d32f2f;
}

.error-icon {
  font-size: 32px;
  margin-bottom: 8px;
}

.error-message {
  margin: 8px 0;
  font-size: 14px;
}

.retry-button {
  background: #2196F3;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: background-color 0.2s;
}

.retry-button:hover {
  background: #1976D2;
}

.edit-window-content {
  max-width: min(90vw, 1000px);
  width: 100%;
  margin: 0 auto;
  background: white;
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

.header {
  text-align: center;
  padding: 24px 20px 16px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.title {
  font-size: 24px;
  font-weight: 700;
  margin: 0 0 8px 0;
}

.instructions {
  font-size: 14px;
  margin: 0 0 16px 0;
  opacity: 0.9;
}

.header-buttons {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.header-button {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  color: white;
}

.header-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.header-button.add-button {
  background: #4CAF50;
}

.header-button.add-button:hover {
  background: #45a049;
}

.header-button.reset-button {
  background: #f44336;
}

.header-button.reset-button:hover {
  background: #da190b;
}

.operations-grid-container {
  padding: 20px;
}

.drag-instructions {
  font-size: 13px;
  color: #666;
  text-align: center;
  margin: 0 0 16px 0;
  font-style: italic;
}

.operations-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 20px;
}

.operation-item {
  position: relative;
  transition: all 0.2s ease;
}

.operation-button {
  background: linear-gradient(135deg, #f3e5f5 0%, #e1bee7 100%);
  border: 2px solid rgba(156, 39, 176, 0.3);
  border-radius: 8px;
  padding: 12px 16px;
  cursor: default;
  transition: all 0.2s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  position: relative;
  min-height: 60px;
  display: flex;
  align-items: center;
  width: 100%;
}

.operation-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
}

.operation-button.chat-operation {
  background: rgba(69, 85, 112, 0.8);
  border-color: rgba(183, 198, 211, 0.3);
}

.reorder-controls {
  position: absolute;
  left: 8px;
  top: 50%;
  transform: translateY(-50%);
  display: flex;
  flex-direction: row;
  gap: 2px;
}

.arrow-button {
  width: 20px;
  height: 20px;
  border: 1px solid #ddd;
  border-radius: 3px;
  background: white;
  color: #666;
  font-size: 10px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  padding: 0;
}

.arrow-button:hover:not(:disabled) {
  background: #f0f0f0;
  border-color: #999;
  color: #333;
  transform: scale(1.1);
}

.arrow-button:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.operation-content {
  flex: 1;
  padding-left: 50px; /* Space for horizontal arrow controls */
  padding-right: 60px; /* Space for icons */
}

.operation-name {
  font-size: 16px;
  font-weight: 600;
  color: #333;
  margin-bottom: 4px;
}

.operation-type {
  font-size: 12px;
  color: #666;
  font-style: italic;
}

.operation-icons {
  position: absolute;
  top: 50%;
  right: 12px;
  transform: translateY(-50%);
  display: flex;
  gap: 8px;
}

.icon-button {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.9);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  transition: all 0.2s ease;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.icon-button:hover {
  background: rgba(255, 255, 255, 1);
  transform: scale(1.1);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
}

.edit-icon:hover {
  background: #e3f2fd;
}

.delete-icon:hover {
  background: #ffebee;
}


/* Dialog Styles */
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.confirm-dialog,
.edit-dialog {
  background: white;
  border-radius: 8px;
  padding: 24px;
  max-width: 90vw;
  max-height: 90vh;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  overflow-y: auto;
}

.confirm-dialog {
  width: 400px;
}

.edit-dialog {
  width: 500px;
}

.dialog-title {
  font-size: 18px;
  font-weight: 600;
  color: #333;
  margin: 0 0 16px 0;
  text-align: center;
}

.dialog-message {
  color: #666;
  margin: 0 0 24px 0;
  text-align: center;
  line-height: 1.4;
}

.dialog-buttons {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.dialog-button {
  padding: 8px 20px;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
  min-width: 80px;
}

.cancel-button {
  background: #e0e0e0;
  color: #333;
}

.cancel-button:hover {
  background: #d0d0d0;
}

.confirm-button {
  background: #f44336;
  color: white;
}

.confirm-button:hover {
  background: #da190b;
}

.save-button {
  background: #4CAF50;
  color: white;
}

.save-button:hover:not(:disabled) {
  background: #45a049;
}

.save-button:disabled {
  background: #ccc;
  cursor: not-allowed;
}

/* Form Styles */
.form-group {
  margin-bottom: 16px;
}

.form-label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: #333;
  margin-bottom: 4px;
}

.form-input,
.form-textarea {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
  box-sizing: border-box;
  transition: border-color 0.2s ease;
}

.form-input:focus,
.form-textarea:focus {
  outline: none;
  border-color: #2196F3;
}

.form-input:disabled {
  background: #f5f5f5;
  color: #999;
}

.form-textarea {
  resize: vertical;
  min-height: 60px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  font-size: 14px;
  color: #333;
  cursor: pointer;
}

.form-checkbox {
  margin-right: 8px;
  width: 16px;
  height: 16px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .edit-window-container {
    background: linear-gradient(135deg, #2d3748 0%, #1a202c 100%);
  }

  .edit-window-content {
    background: rgba(45, 55, 72, 0.9);
  }

  .header {
    background: linear-gradient(135deg, #4a5568 0%, #2d3748 100%);
  }

  .title,
  .instructions {
    color: #e2e8f0;
  }

  .operations-grid-container {
    background: rgba(45, 55, 72, 0.5);
  }

  .operation-button {
    background: rgba(45, 55, 72, 0.8);
    border-color: rgba(255, 255, 255, 0.1);
    color: #e2e8f0;
  }

  .operation-name {
    color: #e2e8f0;
  }

  .operation-type {
    color: #a0aec0;
  }

  .bottom-buttons {
    background: rgba(26, 32, 44, 0.8);
    border-color: rgba(255, 255, 255, 0.1);
  }

  .confirm-dialog,
  .edit-dialog {
    background: #2d3748;
    color: #e2e8f0;
  }

  .dialog-title,
  .dialog-message {
    color: #e2e8f0;
  }

  .form-input,
  .form-textarea {
    background: #4a5568;
    border-color: #718096;
    color: #e2e8f0;
  }

  .form-label,
  .checkbox-label {
    color: #e2e8f0;
  }

  .loading-text {
    color: #a0aec0;
  }

  .drag-instructions {
    color: #a0aec0;
  }
}

/* Responsive adjustments */
@media (max-width: 800px) {
  .edit-window-content {
    max-width: 95vw;
  }
  
  .operations-grid-container {
    padding: 15px;
  }
  
  .operation-content {
    padding-left: 40px;
    padding-right: 50px;
  }
}

@media (max-width: 600px) {
  .edit-dialog {
    width: 90vw;
  }
  
  .header-buttons {
    flex-direction: column;
    align-items: center;
  }
  
  .header-button {
    min-width: 150px;
  }
  
  .operation-content {
    padding-left: 35px;
    padding-right: 45px;
  }
  
  .operation-name {
    font-size: 14px;
  }
  
  .operation-type {
    font-size: 11px;
  }
}
</style>