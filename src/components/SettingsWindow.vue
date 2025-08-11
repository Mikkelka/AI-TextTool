<template>
  <div class="settings-window">
    <h1>Settings</h1>
    <div class="form-section">
      <label>Shortcut Key:</label>
      <input 
        type="text" 
        v-model="formData.shortcut" 
        class="form-input" 
        placeholder="e.g., ctrl+space" 
      />
    </div>
    
    <div class="form-section">
      <h2>Gemini AI</h2>
      <button class="get-api-key-btn" @click="openApiKeyPage">Get API Key</button>
      
      <label>API Key</label>
      <input 
        type="password" 
        v-model="formData.apiKey"
        placeholder="Paste your Gemini API key here" 
        class="form-input" 
      />
      
      <label>Chat Model</label>
      <select class="form-select" v-model="formData.chatModel">
        <option value="gemini-2.5-flash">Gemini 2.5 Flash (most intelligent | fast | 10 uses/min)</option>
        <option value="gemini-2.5-flash-lite">Gemini 2.5 Flash Lite (faster | lightweight | 15 uses/min)</option>
      </select>
      
      <label>Text Operations Model</label>
      <select class="form-select" v-model="formData.textModel">
        <option value="gemini-2.5-flash-lite">Gemini 2.5 Flash Lite (faster | lightweight | 15 uses/min)</option>
        <option value="gemini-2.5-flash">Gemini 2.5 Flash (most intelligent | fast | 10 uses/min)</option>
      </select>
      
      <label>Chat System Instruction</label>
      <textarea 
        class="form-textarea" 
        v-model="formData.systemInstruction"
        placeholder="System instruction for custom chat..."
        rows="4"
      ></textarea>
    </div>
    
    <button class="save-btn" @click="saveSettings" :disabled="isSaving">
      {{ isSaving ? 'Saving...' : 'Save' }}
    </button>
    <p class="restart-notice">Restart required for changes to take effect</p>
    
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

interface Config {
  api_key: string
  chat_system_instruction: string
  provider: string
  chat_model: string
  text_model: string
  shortcut: string
  locale: string
  streaming: boolean
  providers: Record<string, any>
}

const formData = ref({
  shortcut: 'ctrl+space',
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
    const config = await invoke<Config>('load_config')
    
    // Populate form with existing config
    formData.value = {
      shortcut: config.shortcut || 'ctrl+space',
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
    showMessage('Failed to open browser. Please visit https://aistudio.google.com/app/apikey manually.', 'error')
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
      config = await invoke<Config>('load_config')
    } catch {
      // Create default config if none exists
      config = {
        api_key: '',
        chat_system_instruction: '',
        provider: 'Gemini',
        chat_model: 'gemini-2.5-flash',
        text_model: 'gemini-2.5-flash-lite',
        shortcut: 'ctrl+space',
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
    config.shortcut = formData.value.shortcut
    
    // Ensure provider config exists and is updated
    if (!config.providers.Gemini) {
      config.providers.Gemini = {}
    }
    config.providers.Gemini.api_key = formData.value.apiKey
    config.providers.Gemini.chat_model_name = formData.value.chatModel
    config.providers.Gemini.text_model_name = formData.value.textModel
    config.providers.Gemini.chat_system_instruction = formData.value.systemInstruction
    
    // Save the updated configuration
    await invoke('save_config', { config })
    
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
  min-height: 100vh;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

h1 {
  text-align: center;
  margin-bottom: 30px;
  font-size: 24px;
  font-weight: 600;
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

.form-input, .form-select, .form-textarea {
  width: 100%;
  padding: 12px;
  border: 1px solid #444;
  border-radius: 6px;
  background: #2a2a2a;
  color: #ffffff;
  font-size: 14px;
  margin-bottom: 15px;
}

.form-input:focus, .form-select:focus, .form-textarea:focus {
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
</style>