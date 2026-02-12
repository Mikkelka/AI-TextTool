import OnboardingWindow from './components/OnboardingWindow.vue'
import { mountWindow } from './window-bootstrap'

mountWindow(OnboardingWindow, {
  focusSelector: 'input, textarea, button'
})
