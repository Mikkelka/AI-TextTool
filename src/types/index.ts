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
  api_key?: string
  chat_model_name?: string
  text_model_name?: string
  chat_system_instruction?: string
}

export interface Config {
  api_key: string
  chat_system_instruction: string
  provider: string
  chat_model: string
  text_model: string
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
  searchQueries?: string[]
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
export type ModelName = 'gemini-3-flash-preview' | 'gemini-3.1-flash-lite-preview'

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
