<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '../../stores/auth'
import { useRouter } from 'vue-router'
import { Shield, Eye, EyeOff, ArrowRight, Copy, Check } from 'lucide-vue-next'

const authStore = useAuthStore()
const router = useRouter()
const { t } = useI18n()

const step = ref<'password' | 'mnemonic' | 'confirm'>('password')
const password = ref('')
const confirmPassword = ref('')
const showPassword = ref(false)
const error = ref('')
const loading = ref(false)
const copiedIndex = ref<number | null>(null)
const confirmedWords = ref<string>('')
const confirmed = ref(false)

const passwordStrength = computed(() => {
  const p = password.value
  if (!p) return { level: 0, text: '', color: '' }
  let score = 0
  if (p.length >= 8) score++
  if (p.length >= 12) score++
  if (p.length >= 16) score++
  if (/[a-z]/.test(p) && /[A-Z]/.test(p)) score++
  if (/\d/.test(p)) score++
  if (/[^a-zA-Z0-9]/.test(p)) score++

  if (score <= 2) return { level: 1, text: t('passwordGen.weak'), color: 'bg-red-500' }
  if (score <= 3) return { level: 2, text: t('passwordGen.medium'), color: 'bg-yellow-500' }
  if (score <= 4) return { level: 3, text: t('passwordGen.strong'), color: 'bg-green-500' }
  return { level: 4, text: t('passwordGen.veryStrong'), color: 'bg-emerald-500' }
})

async function handleSetPassword() {
  error.value = ''
  if (password.value.length < 8) {
    error.value = t('setup.tooShort')
    return
  }
  if (password.value !== confirmPassword.value) {
    error.value = t('setup.mismatch')
    return
  }
  loading.value = true
  try {
    await authStore.setupMasterPassword(password.value)
    step.value = 'mnemonic'
  } catch (e: any) {
    error.value = t('setup.setupFailed') + e
  } finally {
    loading.value = false
  }
}

async function copyMnemonic() {
  try {
    await navigator.clipboard.writeText(authStore.mnemonicWords.join(' '))
  } catch {
    // Fallback
  }
}

async function copyWord(index: number) {
  copiedIndex.value = index
  try {
    await navigator.clipboard.writeText(authStore.mnemonicWords[index])
  } catch {
    // Fallback
  }
  setTimeout(() => { copiedIndex.value = null }, 1000)
}

function goToConfirm() {
  step.value = 'confirm'
}

async function handleConfirm() {
  if (!confirmed.value) return
  authStore.confirmMnemonicViewed()
  router.push({ name: 'workspaces' })
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center p-4">
    <div class="w-full max-w-lg">
      <!-- Header -->
      <div class="text-center mb-8">
        <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-primary/10 mb-4">
          <Shield class="w-8 h-8 text-primary" />
        </div>
        <h1 class="text-2xl font-bold tracking-tight">{{ t('setup.title') }}</h1>
      </div>

      <!-- Step 1: Set Password -->
      <div v-if="step === 'password'" class="bg-card rounded-xl border border-border p-6 shadow-sm">
        <h2 class="text-lg font-semibold mb-1">{{ t('setup.setPassword') }}</h2>
        <p class="text-sm text-muted-foreground mb-6">{{ t('setup.passwordDesc') }}</p>

        <form @submit.prevent="handleSetPassword" class="space-y-4">
          <div>
            <label class="text-sm font-medium mb-1.5 block">{{ t('setup.passwordLabel') }}</label>
            <div class="relative">
              <input
                v-model="password"
                :type="showPassword ? 'text' : 'password'"
                :placeholder="t('setup.passwordPlaceholder')"
                class="w-full rounded-lg border border-input bg-background px-4 py-3 text-sm
                       placeholder:text-muted-foreground
                       focus:outline-none focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring focus:ring-offset-2"
                autofocus
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
            <!-- Password strength -->
            <div v-if="password" class="mt-2 flex items-center gap-2">
              <div class="flex gap-1 flex-1">
                <div v-for="i in 4" :key="i"
                     class="h-1 flex-1 rounded-full"
                     :class="i <= passwordStrength.level ? passwordStrength.color : 'bg-muted'" />
              </div>
              <span class="text-xs text-muted-foreground">{{ passwordStrength.text }}</span>
            </div>
          </div>

          <div>
            <label class="text-sm font-medium mb-1.5 block">{{ t('setup.confirmLabel') }}</label>
            <input
              v-model="confirmPassword"
              :type="showPassword ? 'text' : 'password'"
              :placeholder="t('setup.confirmPlaceholder')"
              class="w-full rounded-lg border border-input bg-background px-4 py-3 text-sm
                     placeholder:text-muted-foreground
                     focus:outline-none focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring focus:ring-offset-2"
            />
          </div>

          <div v-if="error" class="text-sm text-destructive">{{ error }}</div>

          <button
            type="submit"
            class="w-full rounded-lg bg-primary text-primary-foreground px-4 py-3 text-sm font-medium
                   hover:bg-primary/90 focus:outline-none focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring
                   disabled:cursor-not-allowed disabled:opacity-50 transition-colors
                   flex items-center justify-center gap-2"
            :disabled="loading || !password || !confirmPassword"
          >
            {{ loading ? t('setup.encrypting') : t('setup.setupBtn') }}
            <ArrowRight v-if="!loading" class="w-4 h-4" />
          </button>
        </form>
      </div>

      <!-- Step 2: Show Mnemonic -->
      <div v-if="step === 'mnemonic'" class="bg-card rounded-xl border border-border p-6 shadow-sm">
        <div class="flex items-center gap-2 mb-1">
          <span class="inline-flex items-center justify-center w-6 h-6 rounded-full bg-destructive/10 text-destructive text-xs font-bold">!</span>
          <h2 class="text-lg font-semibold">{{ t('setup.recoveryTitle') }}</h2>
        </div>
        <p class="text-sm text-muted-foreground mb-6">{{ t('setup.recoveryDesc') }}</p>

        <div class="grid grid-cols-3 gap-2 mb-6">
          <div v-for="(word, i) in authStore.mnemonicWords" :key="i"
               class="flex items-center gap-2 px-3 py-2 rounded-lg bg-muted/50 border border-border text-sm group cursor-pointer hover:bg-muted transition-colors"
               @click="copyWord(i)">
            <span class="text-xs text-muted-foreground w-5">{{ i + 1 }}</span>
            <span class="font-mono text-sm">{{ word }}</span>
            <Check v-if="copiedIndex === i" class="w-3 h-3 text-green-500 ml-auto" />
            <Copy v-else class="w-3 h-3 text-muted-foreground opacity-0 group-hover:opacity-100 ml-auto transition-opacity" />
          </div>
        </div>

        <div class="bg-destructive/5 border border-destructive/20 rounded-lg p-3 mb-6">
          <p class="text-xs text-destructive font-medium">{{ t('setup.recoveryWarning') }}</p>
        </div>

        <button
          @click="goToConfirm"
          class="w-full rounded-lg bg-primary text-primary-foreground px-4 py-3 text-sm font-medium
                 hover:bg-primary/90 transition-colors"
        >
          {{ t('setup.recoverySaved') }}
        </button>
      </div>

      <!-- Step 3: Confirm -->
      <div v-if="step === 'confirm'" class="bg-card rounded-xl border border-border p-6 shadow-sm">
        <h2 class="text-lg font-semibold mb-1">{{ t('setup.confirmRecovery') }}</h2>
        <p class="text-sm text-muted-foreground mb-6">{{ t('setup.confirmRecoveryDesc') }}</p>

        <div class="space-y-3">
          <label class="flex items-start gap-3 cursor-pointer">
            <input type="checkbox" v-model="confirmed"
                   class="mt-0.5 rounded border-input" />
            <span class="text-sm">{{ t('setup.confirmCheckbox') }}</span>
          </label>
        </div>

        <button
          @click="handleConfirm"
          class="w-full rounded-lg bg-primary text-primary-foreground px-4 py-3 text-sm font-medium
                 hover:bg-primary/90 transition-colors mt-6
                 disabled:cursor-not-allowed disabled:opacity-50"
          :disabled="!confirmed"
        >
          {{ t('setup.startBtn') }}
        </button>
      </div>
    </div>
  </div>
</template>
