/**
 * Augment the global Window interface with custom properties
 * injected by the Tauri backend via initialization_script()
 */
declare global {
  interface Window {
    /**
     * Clipboard text injected by PopupWindow initialization script
     * Contains the selected text from the system clipboard
     */
    clipboardText?: string
  }
}

export {}
