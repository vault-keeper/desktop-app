<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '../stores/auth'
import { useRouter } from 'vue-router'
import { Shield, Eye, EyeOff, AlertTriangle } from 'lucide-vue-next'

const authStore = useAuthStore()
const router = useRouter()
const { t } = useI18n()

const password = ref('')
const showPassword = ref(false)
const error = ref('')
const loading = ref(false)

onMounted(async () => {
  if (!authStore.isSetupDone) {
    router.replace({ name: 'setup' })
  }
})

async function handleUnlock() {
  if (!password.value) return
  error.value = ''
  loading.value = true
  try {
    const valid = await authStore.verifyMasterPassword(password.value)
    if (valid) {
      router.push({ name: 'workspaces' })
    } else {
      error.value = t('lock.wrongPassword')
    }
  } catch (e) {
    error.value = t('lock.verifyFailed') + e
  } finally {
    loading.value = false
    password.value = ''
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center p-4">
    <div class="w-full max-w-md">
      <!-- Logo & Title -->
      <div class="text-center mb-8">
        <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-primary/10 mb-4">
          <Shield class="w-8 h-8 text-primary" />
        </div>
        <h1 class="text-2xl font-bold tracking-tight">VaultKeeper</h1>
        <p class="text-muted-foreground mt-1">{{ t('lock.subtitle') }}</p>
      </div>

      <!-- Unlock Card -->
      <div class="bg-card rounded-xl border border-border p-6 shadow-sm">
        <form @submit.prevent="handleUnlock" class="space-y-4">
          <div class="relative">
            <input
              v-model="password"
              :type="showPassword ? 'text' : 'password'"
              :placeholder="t('lock.passwordPlaceholder')"
              class="w-full rounded-lg border border-input bg-background px-4 py-3 text-sm
                     placeholder:text-muted-foreground
                     focus:outline-none focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring
                     disabled:cursor-not-allowed disabled:opacity-50"
              :disabled="loading"
              autofocus
              autocomplete="current-password"
            />
            <button
              type="button"
              @click="showPassword = !showPassword"
              class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
            >
              <Eye v-if="!showPassword" class="w-4 h-4" />
              <EyeOff v-else class="w-4 h-4" />
            </button>
          </div>

          <div v-if="error" class="text-sm text-destructive">{{ error }}</div>

          <button
            type="submit"
            class="w-full rounded-lg bg-primary text-primary-foreground px-4 py-3 text-sm font-medium
                   hover:bg-primary/90 focus:outline-none focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring
                   disabled:cursor-not-allowed disabled:opacity-50 transition-colors"
            :disabled="loading || !password"
          >
            {{ loading ? t('lock.verifying') : t('lock.unlock') }}
          </button>
        </form>
      </div>

      <!-- Password loss warning -->
      <div class="flex items-start gap-2 mt-4 px-1">
        <AlertTriangle class="w-4 h-4 text-destructive shrink-0 mt-0.5" />
        <p class="text-xs text-destructive leading-relaxed">{{ t('lock.passwordLossWarning') }}</p>
      </div>

      <!-- Security Notice -->
      <p class="text-xs text-muted-foreground text-center mt-3">
        {{ t('lock.securityNotice') }}
      </p>
    </div>
  </div>
</template>
