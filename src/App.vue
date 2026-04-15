<script setup lang="ts">
import { onMounted, onUnmounted, watch } from 'vue'
import { useAuthStore } from './stores/auth'
import { useRouter } from 'vue-router'
import { listen } from '@tauri-apps/api/event'

const authStore = useAuthStore()
const router = useRouter()

function detectTheme() {
  const saved = localStorage.getItem('vaultkeeper-theme')
  const isDark = saved === 'dark' || (saved !== 'light' && window.matchMedia('(prefers-color-scheme: dark)').matches)
  document.documentElement.classList.toggle('dark', isDark)
}

let autoLockInterval: ReturnType<typeof setInterval> | null = null

function startAutoLockCheck() {
  stopAutoLockCheck()
  autoLockInterval = setInterval(() => {
    if (authStore.shouldAutoLock()) {
      authStore.lockVault()
      router.push({ name: 'lock' })
    }
  }, 10000)
}

function stopAutoLockCheck() {
  if (autoLockInterval) {
    clearInterval(autoLockInterval)
    autoLockInterval = null
  }
}

function onUserActivity() {
  authStore.updateActivity()
}

async function setupTauriListeners() {
  try {
    await listen('tauri://focus', () => {
      if (authStore.shouldAutoLock() && authStore.isUnlocked) {
        authStore.lockVault()
        router.push({ name: 'lock' })
      }
    })
  } catch {
    // Browser mode — no Tauri events
  }
}

onMounted(async () => {
  detectTheme()
  await authStore.checkSetup()

  // Route based on setup state
  if (!authStore.isSetupDone) {
    router.replace({ name: 'setup' })
  } else if (!authStore.isUnlocked) {
    router.replace({ name: 'lock' })
  } else {
    router.replace({ name: 'workspaces' })
  }

  setupTauriListeners()
  if (authStore.isUnlocked) startAutoLockCheck()

  document.addEventListener('mousemove', onUserActivity)
  document.addEventListener('keydown', onUserActivity)
  document.addEventListener('click', onUserActivity)
})

onUnmounted(() => {
  stopAutoLockCheck()
  document.removeEventListener('mousemove', onUserActivity)
  document.removeEventListener('keydown', onUserActivity)
  document.removeEventListener('click', onUserActivity)
})

watch(() => authStore.isUnlocked, (unlocked) => {
  if (unlocked) startAutoLockCheck()
  else stopAutoLockCheck()
})
</script>

<template>
  <div class="min-h-screen bg-background text-foreground font-sans antialiased">
    <router-view v-slot="{ Component }">
      <transition name="fade" mode="out-in">
        <component :is="Component" />
      </transition>
    </router-view>
  </div>
</template>
