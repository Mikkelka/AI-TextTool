import ChatWindow from './components/ChatWindow.vue'
import { mountWindow } from './window-bootstrap'

// Get parameters from URL
const urlParams = new URLSearchParams(window.location.search)
const operation = urlParams.get('operation') || 'Chat'
const title = urlParams.get('title') || 'AI Chat'
const conversationId = urlParams.get('conversationId') || ''

// Check for injected init data (for large text that exceeds URL limits)
const initData = (window as Window & { __chatInitData?: { text: string; instruction: string } })
  .__chatInitData

const initialText = initData?.text ?? urlParams.get('text') ?? ''
const instruction = initData?.instruction ?? urlParams.get('instruction') ?? ''

mountWindow(ChatWindow, {
  props: {
    operation,
    initialText,
    title,
    instruction,
    conversationId
  }
})
