import { ref, onUnmounted } from 'vue'

interface PromptDialogOptions {
  title?: string
  message?: string
  initialValue?: string
  placeholder?: string
  confirmText?: string
  cancelText?: string
}

/**
 * Promise-based prompt dialog state. Call `open()` to show the dialog and
 * await the entered string (trimmed) or `null` if cancelled/empty.
 * Resolves `null` if the component unmounts mid-dialog.
 */
export function usePromptDialog() {
  const visible = ref(false)
  const title = ref('')
  const message = ref('')
  const initialValue = ref('')
  const placeholder = ref('')
  const confirmText = ref('Save')
  const cancelText = ref('Cancel')
  let resolver: ((value: string | null) => void) | null = null

  const open = (options: PromptDialogOptions = {}): Promise<string | null> => {
    title.value = options.title ?? ''
    message.value = options.message ?? ''
    initialValue.value = options.initialValue ?? ''
    placeholder.value = options.placeholder ?? ''
    confirmText.value = options.confirmText ?? 'Save'
    cancelText.value = options.cancelText ?? 'Cancel'
    visible.value = true

    return new Promise<string | null>(resolve => {
      resolver = resolve
    })
  }

  const confirm = (value: string) => {
    visible.value = false
    const trimmed = value.trim()
    resolver?.(trimmed.length > 0 ? trimmed : null)
    resolver = null
  }

  const cancel = () => {
    visible.value = false
    resolver?.(null)
    resolver = null
  }

  onUnmounted(() => {
    if (resolver) {
      resolver(null)
      resolver = null
    }
  })

  return {
    visible,
    title,
    message,
    initialValue,
    placeholder,
    confirmText,
    cancelText,
    open,
    confirm,
    cancel
  }
}
