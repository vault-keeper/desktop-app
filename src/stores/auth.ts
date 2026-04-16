import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useAuthStore = defineStore('auth', () => {
  const isUnlocked = ref(false)
  const isInitialized = ref(false)
  const isSetupDone = ref(false)

  // Auto-lock timer
  const lastActivity = ref(Date.now())
  const autoLockMinutes = ref(5)

  const isSetupComplete = computed(() => isSetupDone.value)

  async function checkSetup() {
    try {
      const [setupDone, unlocked] = await Promise.all([
        invoke<boolean>('is_setup_complete'),
        invoke<boolean>('is_vault_unlocked'),
      ])
      isSetupDone.value = setupDone
      isUnlocked.value = unlocked
    } catch {
      isSetupDone.value = false
      isUnlocked.value = false
    } finally {
      isInitialized.value = true
    }
  }

  async function setupMasterPassword(password: string) {
    await invoke('setup_master_password', { password })
    isUnlocked.value = true
    isSetupDone.value = true
  }

  async function verifyMasterPassword(password: string): Promise<boolean> {
    const valid = await invoke<boolean>('verify_master_password', { password })
    if (valid) {
      isUnlocked.value = true
      lastActivity.value = Date.now()
    }
    return valid
  }

  async function lockVault() {
    await invoke('lock_vault')
    isUnlocked.value = false
  }

  async function changeMasterPassword(currentPassword: string, newPassword: string) {
    await invoke('change_master_password', { currentPassword, newPassword })
  }

  function updateActivity() {
    lastActivity.value = Date.now()
  }

  function shouldAutoLock(): boolean {
    if (!isUnlocked.value) return false
    const elapsed = (Date.now() - lastActivity.value) / 60000
    return elapsed >= autoLockMinutes.value
  }

  function setAutoLockMinutes(minutes: number) {
    autoLockMinutes.value = minutes
  }

  return {
    isUnlocked,
    isInitialized,
    isSetupDone,
    lastActivity,
    autoLockMinutes,
    isSetupComplete,
    checkSetup,
    setupMasterPassword,
    verifyMasterPassword,
    lockVault,
    changeMasterPassword,
    updateActivity,
    shouldAutoLock,
    setAutoLockMinutes,
  }
})
