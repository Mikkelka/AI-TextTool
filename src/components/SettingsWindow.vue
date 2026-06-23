<template>
  <div class="settings-window">
    <div class="header" data-tauri-drag-region>
      <h1 data-tauri-drag-region>Settings</h1>
      <button
        class="close-btn"
        title="Close window"
        data-tauri-drag-region="false"
        @click="closeWindow"
      >
        ✕
      </button>
    </div>

    <div class="form-section">
      <h2>Gemini AI</h2>
      <button class="get-api-key-btn" @click="openApiKeyPage">Get API Key</button>

      <label>API Key</label>
      <input
        v-model="formData.apiKey"
        type="password"
        placeholder="Paste your Gemini API key here"
        class="form-input"
      />

      <label>Chat Model</label>
      <select v-model="formData.chatModel" class="form-select">
        <option v-for="model in MODEL_NAMES" :key="model" :value="model">
          {{ formatModelName(model) }}
        </option>
      </select>

      <label>Text Operations Model</label>
      <select v-model="formData.textModel" class="form-select">
        <option v-for="model in MODEL_NAMES" :key="model" :value="model">
          {{ formatModelName(model) }}
        </option>
      </select>

      <label>Chat System Instruction</label>
      <textarea
        v-model="formData.systemInstruction"
        class="form-textarea"
        placeholder="System instruction for custom chat..."
        rows="4"
      ></textarea>
    </div>

    <button class="save-btn" :disabled="isSaving" @click="saveSettings">
      {{ isSaving ? 'Saving...' : 'Save' }}
    </button>

    <div v-if="message" class="message" :class="messageType">
      {{ message }}
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, onMounted } from 'vue'
  import { invoke } from '@tauri-apps/api/core'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import { logger } from '../utils/logger'
  import type { Config, ModelName } from '../types'
  import { CHAT_MODEL, MODEL_NAMES, TEXT_MODEL, API_KEY_URL, createDefaultConfig } from '../types'
  import { formatModelName } from '../utils/formatters'

  const formData = ref({
    apiKey: '',
    chatModel: CHAT_MODEL,
    textModel: TEXT_MODEL,
    systemInstruction: 'You are a helpful AI assistant.'
  })

  const isSaving = ref(false)
  const message = ref('')
  const messageType = ref<'success' | 'error'>('success')

  const MESSAGE_TIMEOUT_MS = 3000
  const CLOSE_DELAY_MS = 1500

  const showMessage = (text: string, type: 'success' | 'error' = 'success') => {
    message.value = text
    messageType.value = type
    setTimeout(() => {
      message.value = ''
    }, MESSAGE_TIMEOUT_MS)
  }

  const loadConfig = async () => {
    try {
      logger.debug('Loading existing configuration...')
      const config = await invoke<Config>('dm_load_config')

      // Read from active provider
      const active = config.providers?.[config.provider]
      formData.value = {
        apiKey: active?.api_key || '',
        chatModel: (active?.chat_model_name as ModelName) || CHAT_MODEL,
        textModel: (active?.text_model_name as ModelName) || TEXT_MODEL,
        systemInstruction: active?.chat_system_instruction || 'You are a helpful AI assistant.'
      }

      logger.debug('Configuration loaded successfully')
    } catch (error) {
      logger.error('Failed to load config:', error)
      // If no config exists, use defaults (form is already initialized with defaults)
      logger.debug('Using default configuration values')
    }
  }

  const openApiKeyPage = async () => {
    try {
      await openUrl(API_KEY_URL)
      logger.debug('Opened API key page')
    } catch (error) {
      logger.error('Failed to open API key page:', error)
      showMessage(`Failed to open browser. Please visit ${API_KEY_URL} manually.`, 'error')
    }
  }

  const saveSettings = async () => {
    if (isSaving.value) return

    try {
      isSaving.value = true
      logger.debug('Saving settings...')

      // Load existing config first to preserve other settings
      let config: Config
      try {
        config = await invoke<Config>('dm_load_config')
      } catch {
        // Create default config if none exists
        config = createDefaultConfig()
      }

      // Ensure providers object exists (defensive: Rust may omit it)
      if (!config.providers) {
        config.providers = {}
      }

      // Ensure provider config exists and is updated
      if (!config.providers.Gemini) {
        config.providers.Gemini = {
          api_key: '',
          chat_model_name: CHAT_MODEL,
          text_model_name: TEXT_MODEL,
          chat_system_instruction: 'You are a helpful AI assistant.'
        }
      }
      config.providers.Gemini.api_key = formData.value.apiKey
      config.providers.Gemini.chat_model_name = formData.value.chatModel as ModelName
      config.providers.Gemini.text_model_name = formData.value.textModel as ModelName
      config.providers.Gemini.chat_system_instruction = formData.value.systemInstruction

      // Save the updated configuration
      await invoke('dm_save_config', { config })

      showMessage('Settings saved successfully!', 'success')
      logger.debug('Settings saved successfully')

      // Close window after brief delay to show success message
      setTimeout(async () => {
        try {
          const currentWindow = getCurrentWindow()
          await currentWindow.close()
        } catch (error) {
          logger.error('Failed to close window:', error)
        }
      }, CLOSE_DELAY_MS)
    } catch (error) {
      logger.error('Failed to save settings:', error)
      showMessage('Failed to save settings. Please try again.', 'error')
    } finally {
      isSaving.value = false
    }
  }

  const closeWindow = async () => {
    try {
      const currentWindow = getCurrentWindow()
      await currentWindow.close()
    } catch (error) {
      logger.error('Failed to close window:', error)
    }
  }

  onMounted(() => {
    logger.debug('SettingsWindow mounted successfully')
    void loadConfig()
  })
</script>

<style scoped>
  .settings-window {
    padding: var(--space-5);
    background: var(--color-bg-app);
    color: var(--color-text-primary);
    height: 100vh;
    overflow-y: auto;
    overflow-x: hidden;
    box-sizing: border-box;
    font-family: var(--font-family-base);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 30px;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--color-text-tertiary);
    font-size: 18px;
    cursor: pointer;
    padding: var(--space-2);
    border-radius: var(--radius-sm);
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-base);
  }

  .close-btn:hover {
    background: var(--color-border-subtle);
    color: var(--color-text-primary);
  }

  h1 {
    text-align: center;
    margin: 0;
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-semibold);
    flex: 1;
  }

  h2 {
    font-size: var(--font-size-lg);
    margin-bottom: 15px;
    color: var(--color-text-primary);
  }

  .form-section {
    margin-bottom: 25px;
  }

  label {
    display: block;
    margin-bottom: 5px;
    font-size: var(--font-size-base);
    color: var(--color-text-secondary);
  }

  .form-input,
  .form-select,
  .form-textarea {
    width: 100%;
    padding: var(--space-3);
    border: 1px solid var(--input-border);
    border-radius: var(--radius-sm);
    background: var(--input-bg);
    color: var(--input-text);
    font-size: var(--font-size-base);
    margin-bottom: 15px;
    transition:
      border-color var(--transition-base),
      box-shadow var(--transition-base);
  }

  .form-input:focus,
  .form-select:focus,
  .form-textarea:focus {
    outline: none;
    border-color: var(--input-border-focus);
    box-shadow: var(--shadow-focus);
  }

  .form-textarea {
    min-height: 80px;
    resize: vertical;
  }

  .get-api-key-btn {
    background: var(--color-accent);
    color: white;
    border: none;
    padding: var(--space-3) var(--space-5);
    border-radius: var(--radius-sm);
    cursor: pointer;
    margin-bottom: 15px;
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-medium);
    transition: background var(--transition-base);
  }

  .get-api-key-btn:hover {
    background: var(--color-accent-hover);
  }

  .save-btn {
    width: 100%;
    background: var(--color-success);
    color: white;
    border: none;
    padding: 15px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--font-size-md);
    font-weight: var(--font-weight-semibold);
    margin-top: 20px;
    transition: background var(--transition-base);
  }

  .save-btn:hover {
    background: var(--color-success-hover);
  }

  .save-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .message {
    margin-top: 15px;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-sm);
    font-size: var(--font-size-base);
    text-align: center;
    font-weight: var(--font-weight-medium);
  }

  .message.success {
    background: var(--color-success-soft);
    color: var(--color-success);
    border: 1px solid var(--color-success);
  }

  .message.error {
    background: var(--color-danger-soft);
    color: var(--color-danger);
    border: 1px solid var(--color-danger);
  }
</style>
