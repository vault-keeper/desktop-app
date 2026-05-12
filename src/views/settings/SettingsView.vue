<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import { openUrl } from '@tauri-apps/plugin-opener'
import { open, save } from '@tauri-apps/plugin-dialog'
import { useAuthStore } from '../../stores/auth'
import { useWorkspaceStore } from '../../stores/workspace'
import { setLocale, type SupportedLocale } from '../../i18n'
import {
  Monitor, Moon, Sun, Clock, Shield, Key,
  Database, Upload, Download, Languages, RefreshCw, ExternalLink, ArrowLeft
} from 'lucide-vue-next'

const authStore = useAuthStore()
const workspaceStore = useWorkspaceStore()
const router = useRouter()
const { t, locale } = useI18n()

// Bind the dropdown directly to the auth store so the persisted value
// (loaded once in `auth.checkSetup`) is the single source of truth and
// the dropdown can never drift out of sync with what the auto-lock
// checker in App.vue actually uses.
const autoLockMinutes = computed<number>({
  get: () => authStore.autoLockMinutes,
  set: (v) => { authStore.autoLockMinutes = v },
})
const theme = ref<'light' | 'dark' | 'system'>('system')
const appVersion = ref('')
const showChangePassword = ref(false)
const currentPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const passwordError = ref('')
const passwordSuccess = ref(false)
const changingPassword = ref(false)

// ── check for updates ─────────────────────────────────────────
type UpdateState = 'idle' | 'checking' | 'upToDate' | 'hasUpdate' | 'error'
const updateState = ref<UpdateState>('idle')
const latestVersion = ref('')
const downloadUrl = ref('')
const updateError = ref('')

async function checkForUpdates() {
  updateState.value = 'checking'
  updateError.value = ''
  try {
    const result = await invoke<{ current_version: string; latest_version: string; has_update: boolean; download_url: string }>('check_for_updates')
    latestVersion.value = result.latest_version
    downloadUrl.value = result.download_url
    updateState.value = result.has_update ? 'hasUpdate' : 'upToDate'
  } catch (e: any) {
    updateError.value = e.toString()
    updateState.value = 'error'
  }
}

function goBack() {
  const ws = workspaceStore.currentWorkspace
  if (ws) {
    router.push({ name: 'workspace-home', params: { workspaceId: ws.id } })
  } else {
    router.push({ name: 'workspaces' })
  }
}


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
  changingPassword.value = true
  try {
    await invoke('change_master_password', {
      currentPassword: currentPassword.value,
      newPassword: newPassword.value
    })
    passwordSuccess.value = true
    currentPassword.value = ''
    newPassword.value = ''
    confirmPassword.value = ''
    setTimeout(() => {
      showChangePassword.value = false
      passwordSuccess.value = false
    }, 1500)
  } catch (e: any) {
    passwordError.value = e.toString()
  } finally {
    changingPassword.value = false
  }
}

// ── auto lock ─────────────────────────────────────────────────
async function saveAutoLock() {
  // The store has already been updated by v-model on the <select>.
  // Persist the new value to the backend so it survives a restart.
  await invoke('set_auto_lock_timeout', { minutes: authStore.autoLockMinutes })
}

// ── export ────────────────────────────────────────────────────
const showExportModal = ref(false)
const exportPassword = ref('')
const exportPasswordConfirm = ref('')
const exportError = ref('')
const exporting = ref(false)
const exportSuccess = ref(false)

async function startExport() {
  exportError.value = ''
  exportSuccess.value = false
  if (exportPassword.value !== exportPasswordConfirm.value) {
    exportError.value = t('settings.backup.exportPasswordMismatch')
    return
  }
  if (exportPassword.value.length < 1) {
    exportError.value = t('settings.backup.exportPasswordLabel') + ' required'
    return
  }
  const destPath = await save({
    defaultPath: `vaultkeeper-backup-${new Date().toISOString().slice(0, 10).replace(/-/g, '')}.vkbak`,
    filters: [{ name: 'VaultKeeper Backup', extensions: ['vkbak'] }],
  })
  if (!destPath) return
  exporting.value = true
  try {
    await invoke('export_vault', { exportPassword: exportPassword.value, destPath })
    exportSuccess.value = true
    exportPassword.value = ''
    exportPasswordConfirm.value = ''
    setTimeout(() => {
      showExportModal.value = false
      exportSuccess.value = false
    }, 1500)
  } catch (e: any) {
    exportError.value = e.toString()
  } finally {
    exporting.value = false
  }
}

// ── import ────────────────────────────────────────────────────
const showImportModal = ref(false)
const importFilePath = ref('')
const importPassword = ref('')
const importError = ref('')
const importing = ref(false)
const importSuccessCount = ref(0)
const importSuccess = ref(false)

async function pickImportFile() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'VaultKeeper Backup', extensions: ['vkbak'] }],
  })
  if (!selected) return
  importFilePath.value = selected as string
  importError.value = ''
  importSuccess.value = false
  showImportModal.value = true
}

async function startImport() {
  importError.value = ''
  importSuccess.value = false
  importing.value = true
  try {
    const count = await invoke<number>('import_vault', {
      srcPath: importFilePath.value,
      exportPassword: importPassword.value,
    })
    importSuccessCount.value = count
    importSuccess.value = true
    importPassword.value = ''
    setTimeout(() => {
      showImportModal.value = false
      importSuccess.value = false
    }, 1500)
  } catch (e: any) {
    const msg = e.toString()
    importError.value = msg.includes('Incorrect export password')
      ? t('settings.backup.importWrongPassword')
      : msg
  } finally {
    importing.value = false
  }
}

onMounted(async () => {
  try {
    const settings = await invoke<any>('get_settings')
    // auto_lock_minutes is loaded into the auth store by checkSetup,
    // so we only need to pull theme here.
    theme.value = settings.theme || 'system'
  } catch {}
  appVersion.value = await getVersion()
})
</script>

<template>
  <div class="h-screen flex flex-col bg-background overflow-hidden">
    <!-- Top bar -->
    <header class="h-14 shrink-0 border-b border-border bg-card/50 px-6 flex items-center gap-3">
      <button @click="goBack"
              class="p-1.5 rounded-md hover:bg-muted transition-colors text-muted-foreground">
        <ArrowLeft class="w-4 h-4" />
      </button>
      <span class="text-sm font-medium">{{ t('settings.title') }}</span>
    </header>

    <!-- Scrollable content -->
    <div class="flex-1 overflow-y-auto p-6">
      <div class="max-w-2xl mx-auto">

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
        <button @click="showExportModal = true"
                class="flex items-center gap-2 px-4 py-2.5 rounded-lg border border-border
                       text-sm hover:bg-muted transition-colors w-full">
          <Download class="w-4 h-4" />
          {{ t('settings.exportData') }}
        </button>
        <button @click="pickImportFile"
                class="flex items-center gap-2 px-4 py-2.5 rounded-lg border border-border
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

      <!-- Check for updates -->
      <div class="mt-4 flex items-center gap-3 flex-wrap">
        <button @click="checkForUpdates" :disabled="updateState === 'checking'"
                class="flex items-center gap-2 px-4 py-2 rounded-lg border border-border
                       text-sm hover:bg-muted transition-colors disabled:opacity-50">
          <RefreshCw class="w-4 h-4" :class="{ 'animate-spin': updateState === 'checking' }" />
          {{ updateState === 'checking' ? t('settings.updateChecking') : t('settings.checkUpdate') }}
        </button>
        <span v-if="updateState === 'upToDate'" class="text-sm text-green-600 dark:text-green-400">
          {{ t('settings.updateUpToDate') }}
        </span>
        <span v-else-if="updateState === 'error'" class="text-sm text-destructive" :title="updateError">
          {{ t('settings.updateError') }}
        </span>
      </div>
      <div v-if="updateState === 'hasUpdate'"
           class="mt-3 p-3 rounded-lg bg-primary/10 border border-primary/20 flex items-center justify-between gap-3">
        <span class="text-sm text-primary">{{ t('settings.updateAvailable', { version: latestVersion }) }}</span>
        <button @click="openUrl(downloadUrl)"
                class="shrink-0 flex items-center gap-1 text-sm font-medium text-primary hover:underline">
          {{ t('settings.updateDownload') }}
          <ExternalLink class="w-3 h-3" />
        </button>
      </div>
    </section>

    <!-- Change Password Modal -->
    <div v-if="showChangePassword"
         class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
         @click.self="if (!changingPassword) { showChangePassword = false; passwordError = ''; passwordSuccess = false }">
      <div class="bg-card rounded-xl border border-border p-6 w-full max-w-sm shadow-xl">
        <h2 class="text-lg font-semibold mb-4">{{ t('settings.changePasswordModal.title') }}</h2>
        <div class="space-y-3">
          <input v-model="currentPassword" type="password" :placeholder="t('settings.changePasswordModal.currentPlaceholder')" autofocus
                 :disabled="changingPassword"
                 class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                        focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:opacity-50" />
          <input v-model="newPassword" type="password" :placeholder="t('settings.changePasswordModal.newPlaceholder')"
                 :disabled="changingPassword"
                 class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                        focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:opacity-50" />
          <input v-model="confirmPassword" type="password" :placeholder="t('settings.changePasswordModal.confirmPlaceholder')"
                 :disabled="changingPassword"
                 @keyup.enter="changePassword"
                 class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                        focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:opacity-50" />
          <p v-if="passwordError" class="text-sm text-destructive">{{ passwordError }}</p>
          <p v-if="passwordSuccess" class="text-sm text-green-600">{{ t('settings.changePasswordModal.success') }}</p>
          <button @click="changePassword" :disabled="changingPassword || passwordSuccess"
                  class="w-full py-2.5 rounded-lg bg-primary text-primary-foreground text-sm
                         font-medium hover:bg-primary/90 transition-colors disabled:opacity-50">
            {{ changingPassword ? t('common.saving') : t('settings.changePasswordModal.submitBtn') }}
          </button>
          <button @click="showChangePassword = false; passwordError = ''; passwordSuccess = false"
                  :disabled="changingPassword"
                  class="w-full py-2.5 text-sm text-muted-foreground hover:text-foreground disabled:opacity-50">
            {{ t('common.cancel') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Export Modal -->
    <div v-if="showExportModal"
         class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
         @click.self="if (!exporting) { showExportModal = false; exportError = ''; exportPassword = ''; exportPasswordConfirm = '' }">
      <div class="bg-card rounded-xl border border-border p-6 w-full max-w-sm shadow-xl">
        <h2 class="text-lg font-semibold mb-1">{{ t('settings.backup.exportTitle') }}</h2>
        <p class="text-sm text-muted-foreground mb-4">{{ t('settings.backup.exportDesc') }}</p>
        <div class="space-y-3">
          <div>
            <label class="text-xs text-muted-foreground mb-1 block">{{ t('settings.backup.exportPasswordLabel') }}</label>
            <input v-model="exportPassword" type="password" autofocus
                   :disabled="exporting"
                   class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                          focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:opacity-50" />
          </div>
          <div>
            <label class="text-xs text-muted-foreground mb-1 block">{{ t('settings.backup.exportPasswordConfirmLabel') }}</label>
            <input v-model="exportPasswordConfirm" type="password"
                   :disabled="exporting"
                   @keyup.enter="startExport"
                   class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                          focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:opacity-50" />
          </div>
          <p v-if="exportError" class="text-sm text-destructive">{{ exportError }}</p>
          <p v-if="exportSuccess" class="text-sm text-green-600">{{ t('settings.backup.exportSuccess') }}</p>
          <button @click="startExport" :disabled="exporting"
                  class="w-full py-2.5 rounded-lg bg-primary text-primary-foreground text-sm
                         font-medium hover:bg-primary/90 transition-colors disabled:opacity-50">
            {{ exporting ? t('settings.backup.exporting') : t('settings.backup.exportBtn') }}
          </button>
          <button @click="showExportModal = false; exportError = ''; exportPassword = ''; exportPasswordConfirm = ''"
                  :disabled="exporting"
                  class="w-full py-2.5 text-sm text-muted-foreground hover:text-foreground disabled:opacity-50">
            {{ t('common.cancel') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Import Modal -->
    <div v-if="showImportModal"
         class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
         @click.self="if (!importing) { showImportModal = false; importError = ''; importPassword = '' }">
      <div class="bg-card rounded-xl border border-border p-6 w-full max-w-sm shadow-xl">
        <h2 class="text-lg font-semibold mb-1">{{ t('settings.backup.importTitle') }}</h2>
        <p class="text-sm text-muted-foreground mb-4">{{ t('settings.backup.importDesc') }}</p>
        <div class="space-y-3">
          <div>
            <label class="text-xs text-muted-foreground mb-1 block">{{ t('settings.backup.importFile') }}</label>
            <p class="text-sm truncate text-foreground/80 bg-muted rounded px-3 py-2">{{ importFilePath }}</p>
          </div>
          <div>
            <label class="text-xs text-muted-foreground mb-1 block">{{ t('settings.backup.importPasswordLabel') }}</label>
            <input v-model="importPassword" type="password" autofocus
                   :disabled="importing || importSuccess"
                   @keyup.enter="startImport"
                   class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                          focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:opacity-50" />
          </div>
          <p v-if="importError" class="text-sm text-destructive">{{ importError }}</p>
          <p v-if="importSuccess" class="text-sm text-green-600">
            {{ t('settings.backup.importSuccess', { count: importSuccessCount }) }}
          </p>
          <button @click="startImport" :disabled="importing || importSuccess"
                  class="w-full py-2.5 rounded-lg bg-primary text-primary-foreground text-sm
                         font-medium hover:bg-primary/90 transition-colors disabled:opacity-50">
            {{ importing ? t('settings.backup.importing') : t('settings.backup.importBtn') }}
          </button>
          <button @click="showImportModal = false; importError = ''; importPassword = ''"
                  :disabled="importing"
                  class="w-full py-2.5 text-sm text-muted-foreground hover:text-foreground disabled:opacity-50">
            {{ t('common.cancel') }}
          </button>
        </div>
      </div>
    </div>
      </div>
    </div>
  </div>
</template>
