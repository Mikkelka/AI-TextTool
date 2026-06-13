<template>
  <div class="edit-window-container" tabindex="0" @keydown="handleKeydown">
    <!-- Loading State -->
    <div v-if="isLoading" class="loading-container">
      <LoadingSpinner :margin="true" />
      <p class="loading-text">Loading operations...</p>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="error-container">
      <div class="error-icon"><AppIcon :icon="TriangleAlert" :size="30" /></div>
      <p class="error-message">{{ error }}</p>
      <button class="retry-button" @click="loadOperations">Retry</button>
    </div>

    <!-- Main Content -->
    <div v-else>
      <!-- Header -->
      <div class="header" data-tauri-drag-region>
        <div class="header-left" data-tauri-drag-region>
          <h1 class="title" data-tauri-drag-region>Edit AI Text Operations</h1>
          <p class="subtitle" data-tauri-drag-region>
            Click edit/delete icons to modify operations
          </p>
        </div>
        <div class="header-controls">
          <button
            class="control-btn add-btn"
            data-tauri-drag-region="false"
            @click="addNewOperation"
          >
            <AppIcon :icon="Plus" :size="16" />
            Add New
          </button>
          <button
            class="control-btn reset-btn"
            data-tauri-drag-region="false"
            @click="resetToDefaults"
          >
            <AppIcon :icon="RotateCcw" :size="16" />
            Reset to Defaults
          </button>
          <button class="control-btn close-btn" data-tauri-drag-region="false" @click="closeWindow">
            <AppIcon :icon="X" :size="16" />
          </button>
        </div>
      </div>

      <!-- Operations Grid -->
      <div class="operations-grid-container">
        <p class="drag-instructions">
          Use arrow buttons to rearrange order • Click edit/delete icons to modify operations
        </p>
        <div class="operations-grid">
          <div
            v-for="([key, operation], index) in operationsArray"
            :key="key"
            class="operation-item"
          >
            <div class="operation-button" :class="{ 'chat-operation': operation.open_in_window }">
              <div class="reorder-controls">
                <button
                  class="arrow-button"
                  :disabled="index === 0"
                  title="Move up"
                  @click="moveOperation(index, -1)"
                >
                  <AppIcon :icon="ChevronUp" :size="14" />
                </button>
                <button
                  class="arrow-button"
                  :disabled="index === operationsArray.length - 1"
                  title="Move down"
                  @click="moveOperation(index, 1)"
                >
                  <AppIcon :icon="ChevronDown" :size="14" />
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
                  class="icon-button edit-icon"
                  title="Edit operation"
                  @click="editOperation(key)"
                >
                  <AppIcon :icon="Pencil" :size="14" />
                </button>
                <button
                  class="icon-button delete-icon"
                  title="Delete operation"
                  @click="deleteOperation(key)"
                >
                  <AppIcon :icon="Trash2" :size="14" />
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
          <button class="dialog-button cancel-button" @click="cancelConfirm">Cancel</button>
          <button class="dialog-button confirm-button" @click="confirmAction">
            {{ confirmButtonText }}
          </button>
        </div>
      </div>
    </div>

    <!-- Edit Dialog -->
    <div v-if="showEditDialog" class="dialog-overlay" @click="cancelEdit">
      <div class="edit-dialog" @click.stop>
        <h3 class="dialog-title">
          {{ editingOperation ? 'Edit Operation' : 'Add New Operation' }}
        </h3>

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
            <input v-model="editForm.open_in_window" type="checkbox" class="form-checkbox" />
            Open in chat window (instead of direct text replacement)
          </label>
        </div>

        <div class="dialog-buttons">
          <button class="dialog-button cancel-button" @click="cancelEdit">Cancel</button>
          <button
            class="dialog-button save-button"
            :disabled="!editForm.name.trim()"
            @click="saveOperation"
          >
            {{ editingOperation ? 'Update' : 'Add' }} Operation
          </button>
        </div>
      </div>
    </div>

    <AppToast :visible="toastVisible" :message="toastMessage" :type="toastType" />
  </div>
</template>

<script setup lang="ts">
  import { ref, onMounted, onUnmounted } from 'vue'
  import { invoke } from '@tauri-apps/api/core'
  import {
    Pencil,
    Plus,
    RotateCcw,
    Trash2,
    TriangleAlert,
    X,
    ChevronUp,
    ChevronDown
  } from '@lucide/vue'
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import AppIcon from './AppIcon.vue'
  import AppToast from './AppToast.vue'
  import LoadingSpinner from './LoadingSpinner.vue'
  import { logger } from '../utils/logger'
  import type { Operation } from '../types'

  // Props
  interface Props {
    // No props needed for now
  }

  withDefaults(defineProps<Props>(), {})

  // Types

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

  const toastVisible = ref(false)
  const toastMessage = ref('')
  const toastType = ref<'success' | 'error' | 'info'>('info')
  let toastTimer: ReturnType<typeof setTimeout> | null = null

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
      const sortedResult = (await invoke('dm_load_operations_sorted')) as Array<[string, Operation]>
      operationsArray.value = sortedResult

      // Also populate operations object for compatibility
      operations.value = {}
      sortedResult.forEach(([key, operation]) => {
        operations.value[key] = operation
      })

      logger.debug('Loaded operations in order:', sortedResult)
    } catch (err) {
      logger.error('Failed to load operations:', err)
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
    confirmCallback.value = () => void performDelete(operationKey)
    showConfirmDialog.value = true
  }

  const performDelete = async (operationKey: string) => {
    try {
      logger.debug('Attempting to delete operation:', operationKey)
      const success = (await invoke('dm_remove_operation', { name: operationKey })) as boolean
      logger.debug('Delete result:', success)

      // Remove from operationsArray to maintain order
      const indexToRemove = operationsArray.value.findIndex(([key]) => key === operationKey)
      if (indexToRemove !== -1) {
        operationsArray.value.splice(indexToRemove, 1)
      }

      // Also remove from operations object
      delete operations.value[operationKey]

      if (success) {
        logger.debug('Operation deleted successfully:', operationKey)
      } else {
        logger.debug('Backend reported failure, but updating UI anyway')
        // Only reload if backend failed - this preserves order
        await loadOperations()
      }
    } catch (err) {
      logger.error('Failed to delete operation:', err)
      error.value = err instanceof Error ? err.message : 'Failed to delete operation'
      // Reload operations to ensure UI is in sync with backend
      await loadOperations()
    }
  }

  const resetToDefaults = () => {
    confirmTitle.value = 'Reset to Defaults'
    confirmMessage.value =
      'Are you sure you want to reset all operations to their default configuration? This will remove any custom operations you have added.'
    confirmButtonText.value = 'Reset'
    confirmCallback.value = () => void performReset()
    showConfirmDialog.value = true
  }

  const performReset = async () => {
    try {
      await invoke('dm_reset_operations')

      // Reload operations to show the defaults
      await loadOperations()

      logger.debug('Operations reset to defaults successfully')
      showMessage(
        'Reset Complete',
        'All operations have been reset to their default configuration!'
      )
    } catch (err) {
      logger.error('Failed to reset operations:', err)
      error.value = err instanceof Error ? err.message : 'Failed to reset operations'
    }
  }

  const saveOperation = async () => {
    if (!editForm.value.name.trim()) return

    try {
      const operation: Operation = {
        prefix: editForm.value.prefix,
        instruction:
          editForm.value.instruction ||
          "You are a helpful writing assistant. Follow the user's instructions precisely and provide clear, accurate assistance with their text.",
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
      const existingIndex = operationsArray.value.findIndex(
        ([key]) => key === editForm.value.name.trim()
      )
      if (existingIndex !== -1) {
        // Update existing operation
        operationsArray.value[existingIndex] = [editForm.value.name.trim(), operation]
      } else {
        // Add new operation at the end
        operationsArray.value.push([editForm.value.name.trim(), operation])
      }

      logger.debug('Operation saved:', editForm.value.name, operation)
      showEditDialog.value = false

      logger.debug(
        `Operation ${editingOperation.value ? 'updated' : 'added'}:`,
        editForm.value.name
      )
    } catch (err) {
      logger.error('Failed to save operation:', err)
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
      logger.error('Error closing window:', error)
    }
  }

  const handleKeydown = (event: KeyboardEvent) => {
    if (event.key === 'Escape') {
      if (showEditDialog.value) {
        cancelEdit()
      } else if (showConfirmDialog.value) {
        cancelConfirm()
      } else {
        void closeWindow()
      }
    }
  }

  const showToast = (message: string, type: 'success' | 'error' | 'info' = 'info') => {
    toastMessage.value = message
    toastType.value = type
    toastVisible.value = true

    if (toastTimer) {
      clearTimeout(toastTimer)
    }

    toastTimer = setTimeout(() => {
      toastVisible.value = false
    }, 3200)
  }

  const showMessage = (title: string, message: string) => {
    showToast(`${title}: ${message}`, 'success')
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

      logger.debug('Operation moved successfully')
    } catch (err) {
      logger.error('Failed to move operation:', err)
      error.value = err instanceof Error ? err.message : 'Failed to move operation'

      // Reload operations on error to reset to server state
      await loadOperations()
    }
  }

  // Lifecycle
  onMounted(() => {
    void loadOperations()
  })

  onUnmounted(() => {
    if (toastTimer) {
      clearTimeout(toastTimer)
      toastTimer = null
    }
  })
</script>

<style scoped>
  .edit-window-container {
    position: relative;
    width: 100%;
    height: 100vh;
    background: linear-gradient(135deg, var(--color-bg-surface) 0%, var(--color-bg-app) 100%);
    box-sizing: border-box;
    outline: none;
    overflow: auto;
    display: flex;
    flex-direction: column;
    font-family: var(--font-family-base);
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

  .loading-text {
    color: var(--color-text-tertiary);
    font-size: var(--font-size-base);
    margin: 0;
  }

  .error-container {
    color: var(--color-danger);
  }

  .error-icon {
    font-size: 32px;
    margin-bottom: var(--space-2);
  }

  .error-message {
    margin: var(--space-2) 0;
    font-size: var(--font-size-base);
  }

  .retry-button {
    background: var(--color-accent);
    color: white;
    border: none;
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-xs);
    transition: background-color var(--transition-base);
  }

  .retry-button:hover {
    background: var(--color-accent-hover);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-5);
    background: var(--color-bg-surface);
    border-bottom: 1px solid var(--color-border-subtle);
    color: var(--color-text-primary);
  }

  .header-left {
    flex: 1;
  }

  .header-controls {
    display: flex;
    gap: var(--space-2);
    align-items: center;
  }

  .title {
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-bold);
    margin: 0 0 var(--space-2) 0;
  }

  .subtitle {
    font-size: var(--font-size-base);
    margin: 0;
    color: var(--color-text-tertiary);
  }

  .control-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border: none;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    cursor: pointer;
    transition: all var(--transition-base);
    color: white;
    min-height: 32px;
  }

  .control-btn:hover {
    transform: translateY(-1px);
    box-shadow: var(--shadow-sm);
  }

  .add-btn {
    background: var(--color-success);
  }

  .add-btn:hover {
    background: var(--color-success-hover);
  }

  .reset-btn {
    background: var(--color-danger);
  }

  .reset-btn:hover {
    background: var(--color-danger-hover);
  }

  .close-btn {
    background: var(--color-bg-elevated);
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    font-size: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    border: 1px solid var(--color-border);
  }

  .close-btn:hover {
    background: var(--color-border);
  }

  .operations-grid-container {
    padding: var(--space-5);
    background: var(--color-bg-app);
  }

  .drag-instructions {
    font-size: 13px;
    color: var(--color-text-tertiary);
    text-align: center;
    margin: 0 0 var(--space-4) 0;
    font-style: italic;
  }

  .operations-grid {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin-bottom: var(--space-5);
  }

  .operation-item {
    position: relative;
    transition: all var(--transition-base);
  }

  .operation-button {
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: var(--space-3) var(--space-4);
    cursor: default;
    transition: all var(--transition-base);
    box-shadow: var(--shadow-sm);
    position: relative;
    min-height: 60px;
    display: flex;
    align-items: center;
    width: 100%;
  }

  .operation-button:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow-md);
    border-color: var(--color-accent);
  }

  .operation-button.chat-operation {
    background: linear-gradient(135deg, var(--color-accent-soft), rgba(37, 99, 235, 0.2));
    border-color: rgba(59, 130, 246, 0.4);
  }

  .reorder-controls {
    position: absolute;
    left: var(--space-2);
    top: 50%;
    transform: translateY(-50%);
    display: flex;
    flex-direction: row;
    gap: 2px;
  }

  .arrow-button {
    width: 20px;
    height: 20px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-surface);
    color: var(--color-text-tertiary);
    font-size: 10px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-base);
    padding: 0;
  }

  .arrow-button:hover:not(:disabled) {
    background: var(--color-border);
    border-color: var(--color-text-muted);
    color: var(--color-text-primary);
    transform: scale(1.1);
  }

  .arrow-button:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .operation-content {
    flex: 1;
    padding-left: 50px;
    padding-right: 60px;
  }

  .operation-name {
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-primary);
    margin-bottom: var(--space-1);
  }

  .operation-type {
    font-size: var(--font-size-xs);
    color: var(--color-text-tertiary);
    font-style: italic;
  }

  .operation-icons {
    position: absolute;
    top: 50%;
    right: var(--space-3);
    transform: translateY(-50%);
    display: flex;
    gap: var(--space-2);
  }

  .icon-button {
    width: 28px;
    height: 28px;
    border: none;
    border-radius: var(--radius-full);
    background: var(--color-bg-surface);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    transition: all var(--transition-base);
    box-shadow: var(--shadow-sm);
  }

  .icon-button:hover {
    transform: scale(1.1);
    box-shadow: var(--shadow-md);
  }

  .edit-icon {
    color: var(--color-accent);
  }

  .edit-icon:hover {
    background: var(--color-accent-soft);
  }

  .delete-icon {
    color: var(--color-danger);
  }

  .delete-icon:hover {
    background: var(--color-danger-soft);
  }

  /* Dialog Styles */
  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: var(--color-bg-overlay);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .confirm-dialog,
  .edit-dialog {
    background: linear-gradient(145deg, var(--color-bg-elevated), var(--color-bg-app));
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-xl);
    padding: var(--space-6);
    max-width: 90vw;
    max-height: 90vh;
    box-shadow:
      var(--shadow-lg),
      inset 0 1px 0 rgba(255, 255, 255, 0.05);
    overflow-y: auto;
  }

  .confirm-dialog {
    width: 400px;
  }

  .edit-dialog {
    width: 520px;
  }

  .dialog-title {
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-bold);
    color: var(--color-text-primary);
    margin: 0 0 var(--space-2) 0;
    padding-bottom: var(--space-4);
    text-align: center;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .dialog-message {
    color: var(--color-text-secondary);
    margin: var(--space-4) 0 var(--space-6) 0;
    text-align: center;
    line-height: 1.5;
    font-size: var(--font-size-base);
  }

  .dialog-buttons {
    display: flex;
    gap: var(--space-3);
    justify-content: flex-end;
    margin-top: var(--space-6);
    padding-top: var(--space-5);
    border-top: 1px solid var(--color-border-subtle);
  }

  .dialog-button {
    padding: var(--space-2) var(--space-5);
    border: none;
    border-radius: var(--radius-sm);
    font-size: var(--font-size-base);
    cursor: pointer;
    transition: all var(--transition-base);
    min-width: 80px;
  }

  .cancel-button {
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
  }

  .cancel-button:hover {
    background: var(--color-border);
  }

  .confirm-button {
    background: var(--color-danger);
    color: white;
  }

  .confirm-button:hover {
    background: var(--color-danger-hover);
  }

  .save-button {
    background: var(--color-success);
    color: white;
  }

  .save-button:hover:not(:disabled) {
    background: var(--color-success-hover);
  }

  .save-button:disabled {
    background: var(--color-border);
    color: var(--color-text-muted);
    cursor: not-allowed;
  }

  /* Form Styles */
  .form-group {
    margin-bottom: var(--space-5);
  }

  .form-label {
    display: block;
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-semibold);
    color: var(--color-text-primary);
    margin-bottom: var(--space-2);
    letter-spacing: 0.01em;
  }

  .form-input,
  .form-textarea {
    width: 100%;
    padding: var(--space-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    font-size: var(--font-size-base);
    background: var(--color-bg-app);
    color: var(--color-text-primary);
    box-sizing: border-box;
    font-family: inherit;
    transition:
      border-color var(--transition-base),
      box-shadow var(--transition-base),
      background var(--transition-base);
  }

  .form-input::placeholder,
  .form-textarea::placeholder {
    color: var(--color-text-muted);
  }

  .form-input:hover:not(:disabled),
  .form-textarea:hover:not(:disabled) {
    border-color: var(--color-border-strong);
  }

  .form-input:focus,
  .form-textarea:focus {
    outline: none;
    border-color: var(--color-accent);
    box-shadow: var(--shadow-focus);
    background: var(--color-bg-surface);
  }

  .form-input:disabled {
    background: var(--color-bg-surface);
    color: var(--color-text-muted);
    cursor: not-allowed;
  }

  .form-textarea {
    resize: vertical;
    min-height: 80px;
    line-height: 1.5;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--font-size-base);
    color: var(--color-text-primary);
    cursor: pointer;
    padding: var(--space-2) 0;
  }

  .form-checkbox {
    width: 16px;
    height: 16px;
    margin: 0;
    accent-color: var(--color-accent);
  }

  /* Responsive adjustments */
  @media (max-width: 800px) {
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

    .header-controls {
      flex-direction: column;
      gap: 4px;
      align-items: stretch;
    }

    .control-btn {
      min-width: 100px;
    }

    .operation-content {
      padding-left: 35px;
      padding-right: 45px;
    }

    .operation-name {
      font-size: var(--font-size-base);
    }

    .operation-type {
      font-size: 11px;
    }
  }
</style>
