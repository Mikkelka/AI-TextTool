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

export interface ChatHistoryEntry {
  timestamp: string
  original_text: string
  ai_option: string
  processed_text: string
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

export interface AIResponse {
  answer: string
  thoughts?: string
  sources?: GroundingSource[]
  search_queries?: string[]
}

// Error types
export interface AppError {
  message: string
  code?: string
  details?: Record<string, string | number | boolean | null>
}
