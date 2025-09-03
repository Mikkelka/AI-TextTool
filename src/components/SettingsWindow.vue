<template>
  <div class="settings-window">
    <div class="header" data-tauri-drag-region>
      <h1 data-tauri-drag-region>Settings</h1>
      <button class="close-btn" title="Close window" @click="closeWindow" data-tauri-drag-region="false">✕</button>
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
        <option value="gemini-2.5-flash">
          Gemini 2.5 Flash (most intelligent | fast | 10 uses/min)
        </option>
        <option value="gemini-2.5-flash-lite">
          Gemini 2.5 Flash Lite (faster | lightweight | 15 uses/min)
        </option>
      </select>

      <label>Text Operations Model</label>
      <select v-model="formData.textModel" class="form-select">
        <option value="gemini-2.5-flash-lite">
          Gemini 2.5 Flash Lite (faster | lightweight | 15 uses/min)
        </option>
        <option value="gemini-2.5-flash">
          Gemini 2.5 Flash (most intelligent | fast | 10 uses/min)
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
  import type { Config } from '../types'

  const formData = ref({
    apiKey: '',
    chatModel: 'gemini-2.5-flash',
    textModel: 'gemini-2.5-flash-lite',
    systemInstruction: 'You are a helpful AI assistant.'
  })

  const isSaving = ref(false)
  const message = ref('')
  const messageType = ref<'success' | 'error'>('success')

  const showMessage = (text: string, type: 'success' | 'error' = 'success') => {
    message.value = text
    messageType.value = type
    setTimeout(() => {
      message.value = ''
    }, 3000)
  }

  const loadConfig = async () => {
    try {
      console.log('Loading existing configuration...')
      const config = await invoke<Config>('dm_load_config')

      // Populate form with existing config
      formData.value = {
        apiKey: config.api_key || '',
        chatModel: config.chat_model || 'gemini-2.5-flash',
        textModel: config.text_model || 'gemini-2.5-flash-lite',
        systemInstruction: config.chat_system_instruction || 'You are a helpful AI assistant.'
      }

      console.log('Configuration loaded successfully')
    } catch (error) {
      console.error('Failed to load config:', error)
      // If no config exists, use defaults (form is already initialized with defaults)
      console.log('Using default configuration values')
    }
  }

  const openApiKeyPage = async () => {
    try {
      await openUrl('https://aistudio.google.com/app/apikey')
      console.log('Opened API key page')
    } catch (error) {
      console.error('Failed to open API key page:', error)
      showMessage(
        'Failed to open browser. Please visit https://aistudio.google.com/app/apikey manually.',
        'error'
      )
    }
  }

  const saveSettings = async () => {
    if (isSaving.value) return

    try {
      isSaving.value = true
      console.log('Saving settings...')

      // Load existing config first to preserve other settings
      let config: Config
      try {
        config = await invoke<Config>('dm_load_config')
      } catch {
        // Create default config if none exists
        config = {
          api_key: '',
          chat_system_instruction: '',
          provider: 'Gemini',
          chat_model: 'gemini-2.5-flash',
          text_model: 'gemini-2.5-flash-lite',
          shortcut: 'CommandOrControl+Space',
          locale: 'en',
          streaming: false,
          providers: {}
        }
      }

      // Update config with form values
      config.api_key = formData.value.apiKey
      config.chat_system_instruction = formData.value.systemInstruction
      config.chat_model = formData.value.chatModel
      config.text_model = formData.value.textModel

      // Ensure provider config exists and is updated
      if (!config.providers.Gemini) {
        config.providers.Gemini = {}
      }
      config.providers.Gemini.api_key = formData.value.apiKey
      config.providers.Gemini.chat_model_name = formData.value.chatModel
      config.providers.Gemini.text_model_name = formData.value.textModel
      config.providers.Gemini.chat_system_instruction = formData.value.systemInstruction

      // Save the updated configuration
      await invoke('dm_save_config', { config })

      showMessage('Settings saved successfully!', 'success')
      console.log('Settings saved successfully')

      // Close window after brief delay to show success message
      setTimeout(async () => {
        try {
          const currentWindow = getCurrentWindow()
          await currentWindow.close()
        } catch (error) {
          console.error('Failed to close window:', error)
        }
      }, 1500)
    } catch (error) {
      console.error('Failed to save settings:', error)
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
      console.error('Failed to close window:', error)
    }
  }

  onMounted(() => {
    console.log('SettingsWindow mounted successfully')
    loadConfig()
  })
</script>

<style scoped>
  .settings-window {
    padding: 20px;
    background: #1a1a1a;
    color: #ffffff;
    height: 100vh;
    overflow-y: auto;
    overflow-x: hidden;
    box-sizing: border-box;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
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
    color: #ccc;
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
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }

  h1 {
    text-align: center;
    margin: 0;
    font-size: 24px;
    font-weight: 600;
    flex: 1;
  }

  h2 {
    font-size: 18px;
    margin-bottom: 15px;
    color: #ffffff;
  }

  .form-section {
    margin-bottom: 25px;
  }

  label {
    display: block;
    margin-bottom: 5px;
    font-size: 14px;
    color: #cccccc;
  }

  .form-input,
  .form-select,
  .form-textarea {
    width: 100%;
    padding: 12px;
    border: 1px solid #444;
    border-radius: 6px;
    background: #2a2a2a;
    color: #ffffff;
    font-size: 14px;
    margin-bottom: 15px;
  }

  .form-input:focus,
  .form-select:focus,
  .form-textarea:focus {
    outline: none;
    border-color: #4a9eff;
    box-shadow: 0 0 0 2px rgba(74, 158, 255, 0.2);
  }

  .form-textarea {
    min-height: 80px;
    resize: vertical;
  }

  .get-api-key-btn {
    background: #4a9eff;
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 6px;
    cursor: pointer;
    margin-bottom: 15px;
    font-size: 14px;
    font-weight: 500;
  }

  .get-api-key-btn:hover {
    background: #3d8ae6;
  }

  .save-btn {
    width: 100%;
    background: #28a745;
    color: white;
    border: none;
    padding: 15px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 16px;
    font-weight: 600;
    margin-top: 20px;
  }

  .save-btn:hover {
    background: #218838;
  }

  .restart-notice {
    text-align: center;
    color: #888;
    font-size: 12px;
    margin-top: 10px;
  }

  .save-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .message {
    margin-top: 15px;
    padding: 12px 16px;
    border-radius: 6px;
    font-size: 14px;
    text-align: center;
    font-weight: 500;
  }

  .message.success {
    background: #d4edda;
    color: #155724;
    border: 1px solid #c3e6cb;
  }

  .message.error {
    background: #f8d7da;
    color: #721c24;
    border: 1px solid #f5c6cb;
  }

  .form-input.error {
    border-color: #dc3545;
    box-shadow: 0 0 0 2px rgba(220, 53, 69, 0.2);
  }

  .error-message {
    color: #dc3545;
    font-size: 12px;
    margin-top: 5px;
    padding: 5px;
    background: rgba(220, 53, 69, 0.1);
    border-radius: 4px;
    border: 1px solid rgba(220, 53, 69, 0.2);
  }

  .success-message {
    color: #28a745;
    font-size: 12px;
    margin-top: 5px;
    padding: 5px;
    background: rgba(40, 167, 69, 0.1);
    border-radius: 4px;
    border: 1px solid rgba(40, 167, 69, 0.2);
  }

  .suggestions-title {
    margin: 0 0 8px 0;
    font-size: 12px;
    color: #aaa;
    font-weight: 500;
  }

  .suggestion-buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .suggestion-btn {
    padding: 4px 8px;
    background: #333;
    color: #ccc;
    border: 1px solid #555;
    border-radius: 4px;
    font-size: 11px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .suggestion-btn:hover {
    background: #4a9eff;
    color: white;
    border-color: #4a9eff;
  }

  .restart-notice {
    text-align: center;
    color: #ff9800;
    font-size: 13px;
    margin-top: 15px;
    padding: 8px;
    background: rgba(255, 152, 0, 0.1);
    border-radius: 4px;
    border: 1px solid rgba(255, 152, 0, 0.3);
  }
</style>
