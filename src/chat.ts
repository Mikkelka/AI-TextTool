import ChatWindow from './components/ChatWindow.vue'
import { mountWindow } from './window-bootstrap'

// Get parameters from URL
const urlParams = new URLSearchParams(window.location.search)
const operation = urlParams.get('operation') || 'Chat'
const initialText = urlParams.get('text') || ''
const title = urlParams.get('title') || 'AI Chat'
const instruction = urlParams.get('instruction') || ''
const conversationId = urlParams.get('conversationId') || ''

mountWindow(ChatWindow, {
  props: {
    operation,
    initialText,
    title,
    instruction,
    conversationId
  }
})
