// Shared TypeScript interfaces for the AI TextTool application

export interface ChatMessage {
  role: 'user' | 'assistant'
  content: string
  timestamp: string
  isProcessing?: boolean
  thoughts?: string
  sources?: GroundingSource[]
  searchQueries?: string[]
}

export interface GroundingSource {
  title: string
  uri: string
}

export interface Operation {
  prefix: string
  instruction: string
  icon?: string
  open_in_window: boolean
  order?: number
}

// Provider configuration settings
export interface ProviderSettings {
  api_key: string
  chat_model_name: ModelName
  text_model_name: ModelName
  chat_system_instruction: string
}

export interface Config {
  provider: string
  shortcut: string
  locale: string
  streaming: boolean
  providers: Record<string, ProviderSettings>
}

export interface SavedConversation {
  id: string
  title: string
  operation: string
  messages: ConversationMessage[]
  created_at: string
  updated_at: string
  thinking_mode_enabled?: boolean
  grounding_enabled?: boolean
}

export interface ConversationMessage {
  role: 'user' | 'assistant'
  content: string
  timestamp: string
  thoughts?: string
  sources?: GroundingSource[]
  search_queries?: string[]
}

// Props interfaces for components
export interface ChatWindowProps {
  operation?: string
  initialText?: string
  title?: string
  instruction?: string
  conversationId?: string
}

export interface PopupWindowProps {
  selectedText?: string
}

// Utility types
export const CHAT_MODEL = 'gemini-3-flash-preview'
export const TEXT_MODEL = 'gemini-3.1-flash-lite'

export const MODEL_NAMES = [CHAT_MODEL, TEXT_MODEL] as const
export type ModelName = (typeof MODEL_NAMES)[number]

export const MODEL_CAPABILITIES: Record<ModelName, { thinking: boolean; grounding: boolean }> = {
  'gemini-3-flash-preview': { thinking: true, grounding: true },
  'gemini-3.1-flash-lite': { thinking: true, grounding: true }
}

/// Type guard: returns `true` if `value` is one of the supported `ModelName`s.
export function isModelName(value: string): value is ModelName {
  return (MODEL_NAMES as readonly string[]).includes(value)
}

/// Coerce a string to a `ModelName`, falling back to `fallback` if unknown.
export function asModelName(value: string, fallback: ModelName): ModelName {
  return isModelName(value) ? value : fallback
}

export interface AIResponse {
  answer: string
  thoughts?: string
  sources?: GroundingSource[]
  search_queries?: string[]
}

// Chat history entry — mirrors Rust `data_manager::types::ChatEntry`
export interface ChatEntry {
  timestamp: string
  original_text: string
  ai_option: string
  processed_text: string
}

// Default provider name (kept in sync with Rust `Config::default`)
export const DEFAULT_PROVIDER = 'Gemini'
export const DEFAULT_SHORTCUT = 'CmdOrCtrl+Space'
export const DEFAULT_LOCALE = 'en'

// Default Gemini provider settings used when no config exists yet
export function createDefaultProviderSettings(): ProviderSettings {
  return {
    api_key: '',
    chat_model_name: CHAT_MODEL,
    text_model_name: TEXT_MODEL,
    chat_system_instruction: 'You are a helpful AI assistant.'
  }
}

// Default top-level config used when no config exists yet.
// Mirrors Rust `Config::default()` in src-tauri/src/data_manager/types.rs.
export function createDefaultConfig(): Config {
  return {
    provider: DEFAULT_PROVIDER,
    shortcut: DEFAULT_SHORTCUT,
    locale: DEFAULT_LOCALE,
    streaming: false,
    providers: {}
  }
}
