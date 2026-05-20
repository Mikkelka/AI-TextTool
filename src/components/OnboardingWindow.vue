<template>
  <div class="onboarding-window" tabindex="0" @keydown="handleKeydown">
    <!-- Header -->
    <div class="onboarding-header" data-tauri-drag-region>
      <button
        class="close-btn"
        title="Close window"
        data-tauri-drag-region="false"
        @click="closeWindow"
      >
        ✕
      </button>
    </div>

    <!-- Welcome Section -->
    <div class="welcome-section">
      <div class="welcome-icon">🚀</div>
      <h2 class="welcome-title">Welcome to AI Text Tools</h2>
      <p class="welcome-subtitle">Let's get you set up in just a few minutes</p>
    </div>

    <!-- Progress Indicator -->
    <div class="progress-indicator">
      <div class="progress-steps">
        <div
          v-for="(step, index) in steps"
          :key="index"
          :class="[
            'progress-step',
            {
              'progress-step--active': currentStep === index + 1,
              'progress-step--completed': currentStep > index + 1
            }
          ]"
        >
          <div class="step-number">{{ index + 1 }}</div>
          <div class="step-label">{{ step.label }}</div>
        </div>
      </div>
      <div class="progress-bar">
        <div
          class="progress-fill"
          :style="{ width: `${(currentStep / steps.length) * 100}%` }"
        ></div>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="isLoading" class="loading-container">
      <div class="loading-spinner"></div>
      <p class="loading-text">{{ loadingMessage }}</p>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="error-container">
      <div class="error-icon">⚠️</div>
      <p class="error-message">{{ error }}</p>
      <button class="retry-button" @click="clearError">Try Again</button>
    </div>

    <!-- Main Content -->
    <div v-else class="onboarding-content">
      <!-- Step 1: API Key Setup -->
      <div v-if="currentStep === 1" class="setup-step">
        <div class="step-header">
          <h2 class="step-title">🔑 API Key Configuration</h2>
          <p class="step-description">
            You'll need a Google AI Studio API key to use the AI features.
          </p>
        </div>

        <div class="form-group">
          <label for="api-key" class="form-label">
            API Key
            <span class="required">*</span>
          </label>
          <div class="input-with-toggle">
            <input
              id="api-key"
              v-model="formData.apiKey"
              :type="showApiKey ? 'text' : 'password'"
              class="form-input"
              placeholder="Enter your Google AI API key..."
              :class="{ 'form-input--error': errors.apiKey }"
              @blur="validateApiKey"
              @input="clearFieldError('apiKey')"
            />
            <button
              type="button"
              class="toggle-visibility"
              :title="showApiKey ? 'Hide API key' : 'Show API key'"
              @click="showApiKey = !showApiKey"
            >
              {{ showApiKey ? '🙈' : '👁️' }}
            </button>
          </div>
          <div v-if="errors.apiKey" class="field-error">
            {{ errors.apiKey }}
          </div>
          <div class="field-hint">
            Your API key is stored securely and only used for AI requests.
          </div>
        </div>

        <div class="api-key-help">
          <button class="help-button" @click="openApiKeyUrl">
            🌐 Get API Key from Google AI Studio
          </button>
          <p class="help-text">
            Don't have an API key yet? Click above to get one for free from Google AI Studio.
          </p>
        </div>
      </div>

      <!-- Step 2: System Instructions -->
      <div v-if="currentStep === 2" class="setup-step">
        <div class="step-header">
          <h2 class="step-title">🤖 System Instructions</h2>
          <p class="step-description">
            Customize how the AI assistant behaves and responds to your requests.
          </p>
        </div>

        <div class="form-group">
          <label for="system-instruction" class="form-label">System Instruction</label>
          <textarea
            id="system-instruction"
            v-model="formData.systemInstruction"
            class="form-textarea"
            rows="6"
            placeholder="Enter custom system instruction or use the default..."
            @input="clearFieldError('systemInstruction')"
          ></textarea>
          <div class="field-hint">
            This instruction guides the AI's behavior across all operations.
          </div>
        </div>

        <div class="instruction-presets">
          <h3 class="presets-title">Quick Presets:</h3>
          <div class="preset-buttons">
            <button
              class="preset-button"
              :class="{ 'preset-button--active': isActivePreset('default') }"
              @click="setInstructionPreset('default')"
            >
              📝 Helpful Assistant
            </button>
            <button
              class="preset-button"
              :class="{ 'preset-button--active': isActivePreset('concise') }"
              @click="setInstructionPreset('concise')"
            >
              ⚡ Concise & Direct
            </button>
            <button
              class="preset-button"
              :class="{ 'preset-button--active': isActivePreset('professional') }"
              @click="setInstructionPreset('professional')"
            >
              💼 Professional
            </button>
          </div>
        </div>
      </div>

      <!-- Step 3: Connection Test -->
      <div v-if="currentStep === 3" class="setup-step">
        <div class="step-header">
          <h2 class="step-title">🧪 Test Connection</h2>
          <p class="step-description">Let's verify that everything is working correctly.</p>
        </div>

        <div class="test-results">
          <div class="test-item">
            <div class="test-info">
              <div class="test-icon">{{ testResults.apiKey ? '✅' : '❌' }}</div>
              <div class="test-details">
                <div class="test-name">API Key Validation</div>
                <div class="test-status">
                  {{ testResults.apiKey ? 'Valid' : 'Invalid or Empty' }}
                </div>
              </div>
            </div>
          </div>

          <div class="test-item">
            <div class="test-info">
              <div class="test-icon">
                {{
                  testResults.connection === 'testing' ? '⏳' : testResults.connection ? '✅' : '❌'
                }}
              </div>
              <div class="test-details">
                <div class="test-name">AI Service Connection</div>
                <div class="test-status">
                  {{
                    testResults.connection === 'testing'
                      ? 'Testing...'
                      : testResults.connection
                        ? 'Connected'
                        : 'Failed to connect'
                  }}
                </div>
              </div>
            </div>
          </div>
        </div>

        <button
          :disabled="isTesting || !formData.apiKey.trim()"
          class="test-button"
          @click="runConnectionTest"
        >
          {{ isTesting ? 'Testing...' : 'Test Connection' }}
        </button>
      </div>
    </div>

    <!-- Navigation Buttons -->
    <div class="navigation-buttons">
      <button
        v-if="currentStep > 1"
        class="nav-button nav-button--secondary"
        :disabled="isLoading"
        @click="previousStep"
      >
        ← Previous
      </button>

      <div class="nav-spacer"></div>

      <button
        v-if="currentStep < steps.length"
        :disabled="!canProceed || isLoading"
        class="nav-button nav-button--primary"
        @click="nextStep"
      >
        Next →
      </button>

      <button
        v-if="currentStep === steps.length"
        :disabled="!canComplete || isLoading"
        class="nav-button nav-button--success"
        @click="completeSetup"
      >
        🎉 Complete Setup
      </button>
    </div>

    <!-- Skip Setup Option -->
    <div class="skip-setup">
      <button class="skip-button" @click="skipSetup">Skip setup (configure later)</button>
    </div>

    <AppConfirmDialog
      :visible="skipDialogVisible"
      title="Skip Setup"
      message="Are you sure you want to skip setup? You can configure settings later from the system tray menu."
      confirm-text="Skip"
      @confirm="handleSkipDialogConfirm"
      @cancel="handleSkipDialogCancel"
    />
  </div>
</template>

<script setup lang="ts">
  import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
  import { invoke } from '@tauri-apps/api/core'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import AppConfirmDialog from './AppConfirmDialog.vue'
  import { logger } from '../utils/logger'
  import type { Config } from '../types'
  import { CHAT_MODEL, TEXT_MODEL } from '../types'

  // Props
  interface Props {
    visible?: boolean
  }

  withDefaults(defineProps<Props>(), {
    visible: true
  })

  // Emits
  interface Emits {
    (e: 'setup-complete'): void
    (e: 'setup-skipped'): void
    (e: 'close'): void
  }

  const emit = defineEmits<Emits>()

  // Steps configuration
  const steps = [
    { label: 'API Key', id: 'api-key' },
    { label: 'Instructions', id: 'instructions' },
    { label: 'Test', id: 'test' }
  ]

  // Reactive state
  const currentStep = ref(1)
  const isLoading = ref(false)
  const loadingMessage = ref('')
  const error = ref<string | null>(null)
  const showApiKey = ref(false)
  const isTesting = ref(false)
  const skipDialogVisible = ref(false)
  let skipDialogResolver: ((confirmed: boolean) => void) | null = null

  // Form data
  const formData = ref({
    apiKey: '',
    systemInstruction:
      'You are a helpful, friendly AI assistant. Provide clear and accurate assistance with text processing tasks.'
  })

  // Form validation errors
  const errors = ref<Record<string, string>>({})

  // Test results
  const testResults = ref({
    apiKey: false,
    connection: false as boolean | 'testing'
  })

  // System instruction presets
  const instructionPresets = {
    default:
      'You are a helpful, friendly AI assistant. Provide clear and accurate assistance with text processing tasks.',
    concise:
      'You are a concise AI assistant. Provide direct, brief responses without unnecessary elaboration while maintaining accuracy.',
    professional:
      'You are a professional AI writing assistant. Provide polished, formal responses with attention to detail and proper formatting.'
  }

  // Computed properties
  const canProceed = computed(() => {
    switch (currentStep.value) {
      case 1:
        return formData.value.apiKey.trim().length > 0 && !errors.value.apiKey
      case 2:
        return formData.value.systemInstruction.trim().length > 0
      case 3:
        return testResults.value.apiKey && testResults.value.connection === true
      default:
        return true
    }
  })

  const canComplete = computed(() => {
    return testResults.value.apiKey && testResults.value.connection === true
  })

  // Methods
  const validateApiKey = () => {
    const apiKey = formData.value.apiKey.trim()

    if (!apiKey) {
      errors.value.apiKey = 'API key is required'
      testResults.value.apiKey = false
      return false
    }

    if (apiKey.length < 20) {
      errors.value.apiKey = 'API key seems too short'
      testResults.value.apiKey = false
      return false
    }

    if (!apiKey.startsWith('AI') && !apiKey.includes('AI')) {
      errors.value.apiKey = "This doesn't look like a valid Google AI API key"
      testResults.value.apiKey = false
      return false
    }

    delete errors.value.apiKey
    testResults.value.apiKey = true
    return true
  }

  const clearFieldError = (field: string) => {
    if (errors.value[field]) {
      delete errors.value[field]
    }
  }

  const clearError = () => {
    error.value = null
  }

  const closeWindow = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      const currentWindow = getCurrentWindow()
      await currentWindow.close()
    } catch (error) {
      logger.error('Failed to close onboarding window:', error)
    }
  }

  const openApiKeyUrl = async () => {
    try {
      await openUrl('https://aistudio.google.com/app/apikey')
    } catch (err) {
      logger.error('Failed to open API key URL:', err)
      error.value =
        'Failed to open browser. Please visit https://aistudio.google.com/app/apikey manually.'
    }
  }

  const setInstructionPreset = (preset: keyof typeof instructionPresets) => {
    formData.value.systemInstruction = instructionPresets[preset]
  }

  const isActivePreset = (preset: keyof typeof instructionPresets): boolean => {
    return formData.value.systemInstruction === instructionPresets[preset]
  }

  const runConnectionTest = async () => {
    if (!validateApiKey()) {
      return
    }

    isTesting.value = true
    testResults.value.connection = 'testing'
    error.value = null

    try {
      // First save the API key temporarily for testing
      await invoke('dm_update_api_key', { apiKey: formData.value.apiKey })

      // Test the connection
      const isConnected = (await invoke('test_ai_connection')) as boolean

      testResults.value.connection = isConnected

      if (!isConnected) {
        error.value = 'Failed to connect to AI service. Please check your API key.'
      }
    } catch (err) {
      logger.error('Connection test failed:', err)
      testResults.value.connection = false
      error.value = err instanceof Error ? err.message : 'Connection test failed'
    } finally {
      isTesting.value = false
    }
  }

  const nextStep = async () => {
    if (!canProceed.value) return

    if (currentStep.value === 1) {
      // Validate API key before proceeding
      if (!validateApiKey()) {
        return
      }
    }

    if (currentStep.value === 2) {
      // Auto-run connection test when entering step 3
      currentStep.value++
      await runConnectionTest()
      return
    }

    currentStep.value++
  }

  const previousStep = () => {
    if (currentStep.value > 1) {
      currentStep.value--
    }
  }

  const completeSetup = async () => {
    if (!canComplete.value) {
      error.value = 'Please complete all setup steps before finishing.'
      return
    }

    isLoading.value = true
    loadingMessage.value = 'Saving configuration...'
    error.value = null

    try {
      // Load existing config first
      let config: Config
      try {
        config = (await invoke('dm_load_config')) as Config
      } catch {
        // If no config exists, create a default one
        config = {
          api_key: '',
          chat_system_instruction: '',
          provider: 'Gemini',
          chat_model: CHAT_MODEL,
          text_model: TEXT_MODEL,
          shortcut: 'ctrl+space',
          locale: 'en',
          streaming: false,
          providers: {}
        }
      }

      // Update config with form data
      config.api_key = formData.value.apiKey
      config.chat_system_instruction = formData.value.systemInstruction

      // Ensure provider config exists
      if (!config.providers.Gemini) {
        config.providers.Gemini = {
          api_key: formData.value.apiKey,
          chat_model_name: CHAT_MODEL,
          text_model_name: TEXT_MODEL,
          chat_system_instruction: formData.value.systemInstruction
        }
      } else {
        config.providers.Gemini.api_key = formData.value.apiKey
        config.providers.Gemini.chat_system_instruction = formData.value.systemInstruction
      }

      // Save the configuration
      await invoke('dm_save_config', { config })

      loadingMessage.value = 'Setup complete!'

      // Small delay to show completion message, then close window
      setTimeout(async () => {
        isLoading.value = false

        // Close the onboarding window
        try {
          const { getCurrentWindow } = await import('@tauri-apps/api/window')
          const currentWindow = getCurrentWindow()
          await currentWindow.close()
        } catch (error) {
          logger.error('Failed to close onboarding window:', error)
          // Fallback: emit event in case window close fails
          emit('setup-complete')
        }
      }, 1000)
    } catch (err) {
      logger.error('Failed to save configuration:', err)
      error.value = err instanceof Error ? err.message : 'Failed to save configuration'
      isLoading.value = false
    }
  }

  const requestSkipConfirmation = (): Promise<boolean> => {
    skipDialogVisible.value = true
    return new Promise(resolve => {
      skipDialogResolver = resolve
    })
  }

  const handleSkipDialogConfirm = () => {
    skipDialogVisible.value = false
    skipDialogResolver?.(true)
    skipDialogResolver = null
  }

  const handleSkipDialogCancel = () => {
    skipDialogVisible.value = false
    skipDialogResolver?.(false)
    skipDialogResolver = null
  }

  const skipSetup = async () => {
    const shouldSkip = await requestSkipConfirmation()
    if (!shouldSkip) return

    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      const currentWindow = getCurrentWindow()
      await currentWindow.close()
    } catch (error) {
      logger.error('Failed to close onboarding window:', error)
      emit('setup-skipped')
    }
  }

  const handleKeydown = (event: KeyboardEvent) => {
    switch (event.key) {
      case 'Escape':
        void skipSetup()
        break
      case 'Enter':
        if (event.ctrlKey || event.metaKey) {
          if (currentStep.value < steps.length && canProceed.value) {
            void nextStep()
          } else if (currentStep.value === steps.length && canComplete.value) {
            void completeSetup()
          }
        }
        break
      case 'ArrowLeft':
        if (event.altKey && currentStep.value > 1) {
          event.preventDefault()
          previousStep()
        }
        break
      case 'ArrowRight':
        if (event.altKey && currentStep.value < steps.length && canProceed.value) {
          event.preventDefault()
          void nextStep()
        }
        break
    }
  }

  // Auto-validate API key on input
  const watchApiKey = () => {
    if (formData.value.apiKey.trim()) {
      validateApiKey()
    } else {
      // Clear validation error when field becomes empty
      clearFieldError('apiKey')
      testResults.value.apiKey = false
    }
  }

  watch(
    () => formData.value.apiKey,
    () => {
      watchApiKey()
    }
  )

  // Lifecycle
  onMounted(() => {
    // Focus the component for keyboard navigation
    ;(document.querySelector('.onboarding-window') as HTMLElement)?.focus()
  })

  onUnmounted(() => {
    if (skipDialogResolver) {
      skipDialogResolver(false)
      skipDialogResolver = null
    }
  })
</script>

<style scoped>
  .onboarding-window {
    position: relative;
    width: 100%;
    height: 100vh;
    background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
    padding: 40px 20px;
    box-sizing: border-box;
    outline: none;
    overflow-y: auto;
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  /* Header */
  .onboarding-header {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    padding: 12px 20px;
    margin-bottom: 20px;
    width: 100%;
    box-sizing: border-box;
    background: rgba(255, 255, 255, 0.95);
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
    backdrop-filter: blur(10px);
  }

  .close-btn {
    background: none;
    border: none;
    color: #666;
    font-size: 18px;
    cursor: pointer;
    padding: 8px;
    border-radius: 4px;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .close-btn:hover {
    background: rgba(0, 0, 0, 0.1);
    color: #333;
  }

  /* Welcome Section */
  .welcome-section {
    text-align: center;
    margin-bottom: 40px;
    padding: 0 20px;
  }

  .welcome-icon {
    font-size: 64px;
    margin-bottom: 16px;
    display: block;
  }

  .welcome-title {
    font-size: 32px;
    font-weight: 700;
    color: #333;
    margin: 0 0 8px 0;
  }

  .welcome-subtitle {
    font-size: 16px;
    color: #666;
    margin: 0;
  }

  /* Progress Indicator */
  .progress-indicator {
    width: 100%;
    max-width: 500px;
    margin-bottom: 40px;
  }

  .progress-steps {
    display: flex;
    justify-content: space-between;
    margin-bottom: 16px;
  }

  .progress-step {
    display: flex;
    flex-direction: column;
    align-items: center;
    flex: 1;
    opacity: 0.5;
    transition: opacity 0.3s;
  }

  .progress-step--active,
  .progress-step--completed {
    opacity: 1;
  }

  .step-number {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: #ddd;
    color: #666;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
    font-size: 14px;
    margin-bottom: 8px;
    transition: all 0.3s;
  }

  .progress-step--active .step-number {
    background: #2196f3;
    color: white;
  }

  .progress-step--completed .step-number {
    background: #4caf50;
    color: white;
  }

  .step-label {
    font-size: 12px;
    color: #666;
    text-align: center;
  }

  .progress-bar {
    height: 4px;
    background: #e0e0e0;
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #2196f3 0%, #4caf50 100%);
    transition: width 0.5s ease;
    border-radius: 2px;
  }

  /* Loading and Error States */
  .loading-container,
  .error-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px;
    text-align: center;
  }

  .loading-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid #f3f3f3;
    border-top: 3px solid #2196f3;
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
    background: #2196f3;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    transition: background-color 0.2s;
  }

  .retry-button:hover {
    background: #1976d2;
  }

  /* Main Content */
  .onboarding-content {
    width: 100%;
    max-width: 500px;
    background: rgba(255, 255, 255, 0.95);
    border-radius: 16px;
    padding: 32px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
    backdrop-filter: blur(10px);
    margin-bottom: 32px;
  }

  .setup-step {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .step-header {
    text-align: center;
  }

  .step-title {
    font-size: 24px;
    font-weight: 600;
    color: #333;
    margin: 0 0 8px 0;
  }

  .step-description {
    color: #666;
    font-size: 14px;
    margin: 0;
    line-height: 1.5;
  }

  /* Form Elements */
  .form-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .form-label {
    font-weight: 600;
    color: #333;
    font-size: 14px;
  }

  .required {
    color: #d32f2f;
  }

  .input-with-toggle {
    position: relative;
    display: flex;
    align-items: center;
  }

  .form-input {
    flex: 1;
    padding: 12px 16px;
    border: 2px solid #e0e0e0;
    border-radius: 8px;
    font-size: 14px;
    font-family: inherit;
    transition: border-color 0.2s;
    background: white;
  }

  .form-input:focus {
    outline: none;
    border-color: #2196f3;
  }

  .form-input--error {
    border-color: #d32f2f;
  }

  .toggle-visibility {
    position: absolute;
    right: 12px;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 16px;
    padding: 4px;
    border-radius: 4px;
    transition: background-color 0.2s;
  }

  .toggle-visibility:hover {
    background: rgba(0, 0, 0, 0.05);
  }

  .form-textarea {
    padding: 12px 16px;
    border: 2px solid #e0e0e0;
    border-radius: 8px;
    font-size: 14px;
    font-family: inherit;
    resize: vertical;
    min-height: 120px;
    transition: border-color 0.2s;
    background: white;
  }

  .form-textarea:focus {
    outline: none;
    border-color: #2196f3;
  }

  .field-error {
    color: #d32f2f;
    font-size: 12px;
    margin-top: 4px;
  }

  .field-hint {
    color: #666;
    font-size: 12px;
    margin-top: 4px;
  }

  /* API Key Help */
  .api-key-help {
    background: rgba(33, 150, 243, 0.1);
    border: 1px solid rgba(33, 150, 243, 0.2);
    border-radius: 8px;
    padding: 16px;
    text-align: center;
  }

  .help-button {
    background: #2196f3;
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    transition: background-color 0.2s;
    margin-bottom: 8px;
  }

  .help-button:hover {
    background: #1976d2;
  }

  .help-text {
    color: #666;
    font-size: 13px;
    margin: 0;
  }

  /* Instruction Presets */
  .instruction-presets {
    background: #f5f5f5;
    border-radius: 8px;
    padding: 16px;
  }

  .presets-title {
    font-size: 14px;
    font-weight: 600;
    color: #333;
    margin: 0 0 12px 0;
  }

  .preset-buttons {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .preset-button {
    background: white;
    border: 2px solid #e0e0e0;
    border-radius: 6px;
    padding: 12px 16px;
    cursor: pointer;
    font-size: 13px;
    text-align: left;
    transition: all 0.2s;
  }

  .preset-button:hover {
    border-color: #2196f3;
    background: rgba(33, 150, 243, 0.05);
  }

  .preset-button--active {
    border-color: #2196f3;
    background: rgba(33, 150, 243, 0.1);
  }

  /* Test Results */
  .test-results {
    display: flex;
    flex-direction: column;
    gap: 16px;
    margin-bottom: 24px;
  }

  .test-item {
    background: #f8f9fa;
    border-radius: 8px;
    padding: 16px;
  }

  .test-info {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .test-icon {
    font-size: 20px;
    width: 24px;
    text-align: center;
  }

  .test-details {
    flex: 1;
  }

  .test-name {
    font-weight: 600;
    color: #333;
    font-size: 14px;
  }

  .test-status {
    color: #666;
    font-size: 13px;
    margin-top: 2px;
  }

  .test-button {
    width: 100%;
    background: #2196f3;
    color: white;
    border: none;
    padding: 12px 24px;
    border-radius: 8px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    transition: background-color 0.2s;
  }

  .test-button:hover:not(:disabled) {
    background: #1976d2;
  }

  .test-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* Navigation */
  .navigation-buttons {
    display: flex;
    align-items: center;
    gap: 16px;
    width: 100%;
    max-width: 500px;
    margin-bottom: 16px;
  }

  .nav-spacer {
    flex: 1;
  }

  .nav-button {
    padding: 12px 24px;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    transition: all 0.2s;
    min-width: 120px;
  }

  .nav-button--primary {
    background: #2196f3;
    color: white;
  }

  .nav-button--primary:hover:not(:disabled) {
    background: #1976d2;
  }

  .nav-button--secondary {
    background: rgba(255, 255, 255, 0.8);
    color: #333;
    border: 1px solid #e0e0e0;
  }

  .nav-button--secondary:hover:not(:disabled) {
    background: white;
    border-color: #2196f3;
  }

  .nav-button--success {
    background: #4caf50;
    color: white;
  }

  .nav-button--success:hover:not(:disabled) {
    background: #45a049;
  }

  .nav-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* Skip Setup */
  .skip-setup {
    text-align: center;
  }

  .skip-button {
    background: none;
    border: none;
    color: #666;
    cursor: pointer;
    font-size: 13px;
    text-decoration: underline;
    transition: color 0.2s;
  }

  .skip-button:hover {
    color: #333;
  }

  /* Animations */
  @keyframes spin {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }

  /* Dark mode support */
  @media (prefers-color-scheme: dark) {
    .onboarding-window {
      background: linear-gradient(135deg, #2d3748 0%, #1a202c 100%);
    }

    .onboarding-header {
      background: rgba(45, 55, 72, 0.95);
      border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    }

    .close-btn {
      color: #a0aec0;
    }

    .close-btn:hover {
      background: rgba(255, 255, 255, 0.1);
      color: #e2e8f0;
    }

    .welcome-title {
      color: #e2e8f0;
    }

    .welcome-subtitle {
      color: #a0aec0;
    }

    .step-label {
      color: #a0aec0;
    }

    .step-title {
      color: #e2e8f0;
    }

    .step-description {
      color: #cbd5e0;
    }

    .onboarding-content {
      background: rgba(45, 55, 72, 0.95);
      color: #e2e8f0;
    }

    .form-label {
      color: #e2e8f0;
    }

    .form-input,
    .form-textarea {
      background: #4a5568;
      border-color: #2d3748;
      color: #e2e8f0;
    }

    .form-input:focus,
    .form-textarea:focus {
      border-color: #3182ce;
    }

    .field-hint {
      color: #a0aec0;
    }

    .api-key-help {
      background: rgba(59, 130, 246, 0.1);
      border-color: rgba(59, 130, 246, 0.2);
    }

    .help-text {
      color: #a0aec0;
    }

    .instruction-presets {
      background: #2d3748;
    }

    .presets-title {
      color: #e2e8f0;
    }

    .preset-button {
      background: #4a5568;
      border-color: #2d3748;
      color: #e2e8f0;
    }

    .preset-button:hover {
      border-color: #3182ce;
      background: rgba(49, 130, 206, 0.1);
    }

    .preset-button--active {
      border-color: #3182ce;
      background: rgba(49, 130, 206, 0.2);
    }

    .test-item {
      background: #2d3748;
    }

    .test-name {
      color: #e2e8f0;
    }

    .test-status {
      color: #cbd5e0;
    }

    .nav-button--secondary {
      background: rgba(45, 55, 72, 0.8);
      color: #e2e8f0;
      border-color: #2d3748;
    }

    .nav-button--secondary:hover:not(:disabled) {
      background: #4a5568;
      border-color: #3182ce;
    }

    .skip-button {
      color: #a0aec0;
    }

    .skip-button:hover {
      color: #cbd5e0;
    }

    .loading-text {
      color: #a0aec0;
    }
  }

  /* Responsive */
  @media (max-width: 768px) {
    .onboarding-window {
      padding: 0;
    }

    .welcome-icon {
      font-size: 48px;
    }

    .welcome-title {
      font-size: 24px;
    }

    .onboarding-content {
      padding: 24px 20px;
    }

    .step-title {
      font-size: 20px;
    }

    .preset-buttons {
      gap: 6px;
    }

    .navigation-buttons {
      flex-direction: column;
      gap: 12px;
    }

    .nav-spacer {
      display: none;
    }

    .nav-button {
      width: 100%;
    }
  }

  @media (max-width: 480px) {
    .progress-steps {
      justify-content: space-around;
    }

    .step-number {
      width: 28px;
      height: 28px;
      font-size: 12px;
    }

    .step-label {
      font-size: 11px;
    }

    .onboarding-content {
      padding: 20px 16px;
    }
  }
</style>
