import { ref, onUnmounted } from 'vue'

type ToastType = 'success' | 'error' | 'info'

const TOAST_TIMEOUT_MS = 3200

/**
 * Reactive toast notification state with auto-dismiss timer.
 * Cleans up the timer on unmount.
 */
export function useToast() {
  const visible = ref(false)
  const message = ref('')
  const type = ref<ToastType>('info')
  let timer: ReturnType<typeof setTimeout> | null = null

  const show = (msg: string, t: ToastType = 'info') => {
    message.value = msg
    type.value = t
    visible.value = true

    if (timer) {
      clearTimeout(timer)
    }

    timer = setTimeout(() => {
      visible.value = false
    }, TOAST_TIMEOUT_MS)
  }

  onUnmounted(() => {
    if (timer) {
      clearTimeout(timer)
      timer = null
    }
    visible.value = false
  })

  return { visible, message, type, show }
}
