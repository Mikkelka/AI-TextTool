/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

// Global Window interface extensions
declare global {
  interface Window {
    clipboardText?: string
  }
}
