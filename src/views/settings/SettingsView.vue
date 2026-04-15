<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import { useAuthStore } from '../../stores/auth'
import { setLocale, type SupportedLocale } from '../../i18n'
import {
  Monitor, Moon, Sun, Clock, Shield, Key,
  Database, Upload, Download, Languages
} from 'lucide-vue-next'

const authStore = useAuthStore()
const { t, locale } = useI18n()

const autoLockMinutes = ref(5)
const theme = ref<'light' | 'dark' | 'system'>('system')
const appVersion = ref('')
const showChangePassword = ref(false)
const currentPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const passwordError = ref('')
const passwordSuccess = ref(false)

// ── theme ─────────────────────────────────────────────────────
function applyTheme(t: 'light' | 'dark' | 'system') {
  theme.value = t
  localStorage.setItem('vaultkeeper-theme', t)
  if (t === 'dark') {
    document.documentElement.classList.add('dark')
  } else if (t === 'light') {
    document.documentElement.classList.remove('dark')
  } else {
    document.documentElement.classList.toggle('dark', window.matchMedia('(prefers-color-scheme: dark)').matches)
  }
}

// ── language ──────────────────────────────────────────────────
function switchLocale(lang: SupportedLocale) {
  setLocale(lang)
}

// ── password ──────────────────────────────────────────────────
async function changePassword() {
  passwordError.value = ''
  passwordSuccess.value = false
  if (newPassword.value !== confirmPassword.value) {
    passwordError.value = t('settings.changePasswordModal.mismatch')
    return
  }
  if (newPassword.value.length < 8) {
    passwordError.value = t('settings.changePasswordModal.tooShort')
    return
  }
  try {
    await invoke('change_master_password', {
      currentPassword: currentPassword.value,
      newPassword: newPassword.value
    })
    passwordSuccess.value = true
    currentPassword.value = ''
    newPassword.value = ''
    confirmPassword.value = ''
  } catch (e: any) {
    passwordError.value = e.toString()
  }
}

// ── auto lock ─────────────────────────────────────────────────
async function saveAutoLock() {
  await invoke('set_auto_lock_timeout', { minutes: autoLockMinutes.value })
  authStore.autoLockMinutes = autoLockMinutes.value
}

// ── init ──────────────────────────────────────────────────────
onMounted(async () => {
  try {
    const settings = await invoke<any>('get_settings')
    autoLockMinutes.value = settings.auto_lock_minutes || 5
    theme.value = settings.theme || 'system'
  } catch {}
  appVersion.value = await getVersion()
})
</script>

<template>
  <div class="max-w-2xl mx-auto">
    <h1 class="text-2xl font-bold mb-6">{{ t('settings.title') }}</h1>

    <!-- Appearance -->
    <section class="rounded-xl border border-border bg-card p-6 mb-4">
      <h2 class="font-semibold mb-4 flex items-center gap-2">
        <Monitor class="w-4 h-4" />
        {{ t('settings.appearance') }}
      </h2>
      <div class="flex gap-2">
        <button
          v-for="opt in [
            { v: 'light',  label: t('settings.themeLight'),  icon: Sun     },
            { v: 'dark',   label: t('settings.themeDark'),   icon: Moon    },
            { v: 'system', label: t('settings.themeSystem'), icon: Monitor }
          ]"
          :key="opt.v"
          @click="applyTheme(opt.v as any)"
          class="flex items-center gap-2 px-4 py-2.5 rounded-lg border text-sm transition-colors"
          :class="theme === opt.v
            ? 'border-primary bg-primary/10 text-primary'
            : 'border-border hover:bg-muted'">
          <component :is="opt.icon" class="w-4 h-4" />
          {{ opt.label }}
        </button>
      </div>
    </section>

    <!-- Language -->
    <section class="rounded-xl border border-border bg-card p-6 mb-4">
      <h2 class="font-semibold mb-4 flex items-center gap-2">
        <Languages class="w-4 h-4" />
        {{ t('settings.language') }}
      </h2>
      <div class="flex gap-2">
        <button
          v-for="lang in [
            { v: 'zh-CN', label: '简体中文' },
            { v: 'en-US', label: 'English'  }
          ]"
          :key="lang.v"
          @click="switchLocale(lang.v as SupportedLocale)"
          class="px-4 py-2.5 rounded-lg border text-sm transition-colors"
          :class="locale === lang.v
            ? 'border-primary bg-primary/10 text-primary'
            : 'border-border hover:bg-muted'">
          {{ lang.label }}
        </button>
      </div>
    </section>

    <!-- Auto Lock -->
    <section class="rounded-xl border border-border bg-card p-6 mb-4">
      <h2 class="font-semibold mb-4 flex items-center gap-2">
        <Clock class="w-4 h-4" />
        {{ t('settings.autoLock') }}
      </h2>
      <div class="flex items-center gap-3">
        <select v-model="autoLockMinutes" @change="saveAutoLock"
                class="px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                       focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
          <option :value="1">{{ t('settings.lockOptions.1') }}</option>
          <option :value="3">{{ t('settings.lockOptions.3') }}</option>
          <option :value="5">{{ t('settings.lockOptions.5') }}</option>
          <option :value="10">{{ t('settings.lockOptions.10') }}</option>
          <option :value="15">{{ t('settings.lockOptions.15') }}</option>
          <option :value="30">{{ t('settings.lockOptions.30') }}</option>
        </select>
        <span class="text-sm text-muted-foreground">{{ t('settings.autoLockDesc') }}</span>
      </div>
    </section>

    <!-- Security -->
    <section class="rounded-xl border border-border bg-card p-6 mb-4">
      <h2 class="font-semibold mb-4 flex items-center gap-2">
        <Shield class="w-4 h-4" />
        {{ t('settings.security') }}
      </h2>
      <button @click="showChangePassword = true"
              class="flex items-center gap-2 px-4 py-2.5 rounded-lg border border-border
                     text-sm hover:bg-muted transition-colors">
        <Key class="w-4 h-4" />
        {{ t('settings.changePassword') }}
      </button>
    </section>

    <!-- Data -->
    <section class="rounded-xl border border-border bg-card p-6 mb-4">
      <h2 class="font-semibold mb-4 flex items-center gap-2">
        <Database class="w-4 h-4" />
        {{ t('settings.dataManagement') }}
      </h2>
      <div class="space-y-2">
        <button class="flex items-center gap-2 px-4 py-2.5 rounded-lg border border-border
                       text-sm hover:bg-muted transition-colors w-full">
          <Download class="w-4 h-4" />
          {{ t('settings.exportData') }}
        </button>
        <button class="flex items-center gap-2 px-4 py-2.5 rounded-lg border border-border
                       text-sm hover:bg-muted transition-colors w-full">
          <Upload class="w-4 h-4" />
          {{ t('settings.importData') }}
        </button>
      </div>
    </section>

    <!-- About -->
    <section class="rounded-xl border border-border bg-card p-6">
      <h2 class="font-semibold mb-2">{{ t('settings.about') }}</h2>
      <p class="text-sm text-muted-foreground">{{ t('settings.version') }} {{ appVersion }}</p>
      <p class="text-xs text-muted-foreground mt-1">{{ t('settings.securityInfo') }}</p>
    </section>

    <!-- Change Password Modal -->
    <div v-if="showChangePassword"
         class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
         @click.self="showChangePassword = false; passwordError = ''; passwordSuccess = false">
      <div class="bg-card rounded-xl border border-border p-6 w-full max-w-sm shadow-xl">
        <h2 class="text-lg font-semibold mb-4">{{ t('settings.changePasswordModal.title') }}</h2>
        <div class="space-y-3">
          <input v-model="currentPassword" type="password" :placeholder="t('settings.changePasswordModal.currentPlaceholder')" autofocus
                 class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                        focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
          <input v-model="newPassword" type="password" :placeholder="t('settings.changePasswordModal.newPlaceholder')"
                 class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                        focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
          <input v-model="confirmPassword" type="password" :placeholder="t('settings.changePasswordModal.confirmPlaceholder')"
                 class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                        focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
          <p v-if="passwordError" class="text-sm text-destructive">{{ passwordError }}</p>
          <p v-if="passwordSuccess" class="text-sm text-green-600">{{ t('settings.changePasswordModal.success') }}</p>
          <button @click="changePassword"
                  class="w-full py-2.5 rounded-lg bg-primary text-primary-foreground text-sm
                         font-medium hover:bg-primary/90 transition-colors">
            {{ t('settings.changePasswordModal.submitBtn') }}
          </button>
          <button @click="showChangePassword = false; passwordError = ''; passwordSuccess = false"
                  class="w-full py-2.5 text-sm text-muted-foreground hover:text-foreground">
            {{ t('common.cancel') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
