import OperationEditWindow from './components/OperationEditWindow.vue'
import { mountWindow } from './window-bootstrap'

mountWindow(OperationEditWindow, {
  focusSelector: '.edit-window-container'
})
