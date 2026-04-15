import { onMounted, onUnmounted } from 'vue'
import { useAuthStore } from '../stores/auth'

export function useAutoLock(timeoutMinutes: number = 5) {
  const authStore = useAuthStore()
  let interval: ReturnType<typeof setInterval> | null = null

  function startChecking() {
    stopChecking()
    interval = setInterval(() => {
      authStore.autoLockMinutes = timeoutMinutes
      if (authStore.shouldAutoLock()) {
        authStore.lockVault()
      }
    }, 10000)
  }

  function stopChecking() {
    if (interval) {
      clearInterval(interval)
      interval = null
    }
  }

  function updateTimeout(minutes: number) {
    authStore.autoLockMinutes = minutes
  }

  onMounted(() => {
    if (authStore.isUnlocked) {
      startChecking()
    }
  })

  onUnmounted(() => {
    stopChecking()
  })

  return {
    startChecking,
    stopChecking,
    updateTimeout,
  }
}