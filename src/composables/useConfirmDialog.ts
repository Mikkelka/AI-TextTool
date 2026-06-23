import { ref, onUnmounted } from 'vue'

interface ConfirmDialogOptions {
  title?: string
  message?: string
  confirmText?: string
  cancelText?: string
  danger?: boolean
}

/**
 * Promise-based confirm dialog state. Call `open()` to show the dialog and
 * await a boolean result. Resolves `false` if the component unmounts mid-dialog.
 */
export function useConfirmDialog() {
  const visible = ref(false)
  const title = ref('')
  const message = ref('')
  const confirmText = ref('Confirm')
  const cancelText = ref('Cancel')
  const danger = ref(false)
  let resolver: ((confirmed: boolean) => void) | null = null

  const open = (options: ConfirmDialogOptions = {}): Promise<boolean> => {
    title.value = options.title ?? ''
    message.value = options.message ?? ''
    confirmText.value = options.confirmText ?? 'Confirm'
    cancelText.value = options.cancelText ?? 'Cancel'
    danger.value = options.danger ?? false
    visible.value = true

    return new Promise<boolean>(resolve => {
      resolver = resolve
    })
  }

  const confirm = () => {
    visible.value = false
    resolver?.(true)
    resolver = null
  }

  const cancel = () => {
    visible.value = false
    resolver?.(false)
    resolver = null
  }

  onUnmounted(() => {
    if (resolver) {
      resolver(false)
      resolver = null
    }
  })

  return { visible, title, message, confirmText, cancelText, danger, open, confirm, cancel }
}
