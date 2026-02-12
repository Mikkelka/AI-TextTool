import PopupWindow from './components/PopupWindow.vue'
import { mountWindow } from './window-bootstrap'

mountWindow(PopupWindow, {
  props: {
    selectedText: window.clipboardText || ''
  }
})
