import SettingsWindow from './components/SettingsWindow.vue'
import { mountWindow } from './window-bootstrap'

mountWindow(SettingsWindow, {
  focusSelector: 'input'
})
