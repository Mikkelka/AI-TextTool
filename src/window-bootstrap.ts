import { createApp, type App, type Component } from 'vue'

interface WindowBootstrapOptions {
  props?: Record<string, unknown>
  focusSelector?: string
  focusDelayMs?: number
}

export function mountWindow(
  component: Component,
  { props = {}, focusSelector, focusDelayMs = 100 }: WindowBootstrapOptions = {}
): App {
  const app = createApp(component, props)
  app.mount('#app')

  if (focusSelector) {
    setTimeout(() => {
      document.querySelector<HTMLElement>(focusSelector)?.focus()
    }, focusDelayMs)
  }

  return app
}
