<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { X, Copy, Check, RefreshCw } from 'lucide-vue-next'

const emit = defineEmits<{
  close: []
  select: [password: string]
}>()

const { t } = useI18n()
const length = ref(16)
const uppercase = ref(true)
const lowercase = ref(true)
const numbers = ref(true)
const symbols = ref(true)
const excludeAmbiguous = ref(false)

const generated = ref('')
const copied = ref(false)
const strength = ref({ level: 0, text: '', color: '' })

const strengthText = computed((): { text: string; color: string } => {
  if (!generated.value) return { text: '', color: '' }
  const s = strength.value
  if (s.level <= 1) return { text: t('passwordGen.weak'), color: 'bg-red-500' }
  if (s.level === 2) return { text: t('passwordGen.medium'), color: 'bg-yellow-500' }
  if (s.level === 3) return { text: t('passwordGen.strong'), color: 'bg-green-500' }
  return { text: t('passwordGen.veryStrong'), color: 'bg-emerald-500' }
})

function updateStrength(pwd: string) {
  if (!pwd) {
    strength.value = { level: 0, text: '', color: '' }
    return
  }
  let score = 0
  if (pwd.length >= 8) score++
  if (pwd.length >= 12) score++
  if (pwd.length >= 16) score++
  if (/[a-z]/.test(pwd) && /[A-Z]/.test(pwd)) score++
  if (/\d/.test(pwd)) score++
  if (/[^a-zA-Z0-9]/.test(pwd)) score++
  strength.value = { level: Math.min(score, 4), text: '', color: '' }
}

async function generate() {
  try {
    const pwd = await invoke<string>('generate_password', {
      length: length.value,
      options: {
        uppercase: uppercase.value,
        lowercase: lowercase.value,
        numbers: numbers.value,
        symbols: symbols.value,
        exclude_ambiguous: excludeAmbiguous.value,
      }
    })
    generated.value = pwd
    updateStrength(pwd)
  } catch {
    // Fallback: generate in JS
    let charset = ''
    if (lowercase.value) charset += 'abcdefghijklmnopqrstuvwxyz'
    if (uppercase.value) charset += 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'
    if (numbers.value) charset += '0123456789'
    if (symbols.value) charset += '!@#$%^&*()_+-=[]{}|;:,.<>?'
    if (excludeAmbiguous.value) charset = charset.replace(/[0OlI1|]/g, '')
    if (!charset) charset = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'

    let pwd = ''
    for (let i = 0; i < length.value; i++) {
      pwd += charset[Math.floor(Math.random() * charset.length)]
    }
    generated.value = pwd
    updateStrength(pwd)
  }
}

async function copyToClipboard() {
  if (!generated.value) return
  try {
    await navigator.clipboard.writeText(generated.value)
    copied.value = true
    setTimeout(() => { copied.value = false }, 1500)
  } catch {
    // Fallback
  }
}

function usePassword() {
  if (generated.value) {
    emit('select', generated.value)
  }
}

watch([length, uppercase, lowercase, numbers, symbols, excludeAmbiguous], () => {
  if (generated.value) generate()
})

generate()
</script>

<template>
  <div class="bg-card rounded-xl border border-border p-6 w-full max-w-sm shadow-xl">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-semibold">{{ t('passwordGen.title') }}</h2>
      <button @click="emit('close')" class="p-1.5 rounded-md hover:bg-muted transition-colors">
        <X class="w-4 h-4" />
      </button>
    </div>

    <!-- Generated Password -->
    <div class="bg-muted/50 rounded-lg p-3 mb-4">
      <div class="flex items-center gap-2">
        <code class="flex-1 text-sm font-mono break-all select-all">{{ generated || t('passwordGen.clickGenerate') }}</code>
        <button @click="copyToClipboard" class="p-1.5 rounded hover:bg-muted shrink-0" :title="t('passwordGen.copy')">
          <Check v-if="copied" class="w-4 h-4 text-green-500" />
          <Copy v-else class="w-4 h-4 text-muted-foreground" />
        </button>
      </div>
      <div v-if="generated" class="mt-2 flex items-center gap-2">
        <div class="flex gap-1 flex-1">
          <div v-for="i in 4" :key="i"
               class="h-1 flex-1 rounded-full"
               :class="i <= strength.level ? strengthText.color : 'bg-muted'" />
        </div>
        <span class="text-xs text-muted-foreground">{{ strengthText.text }}</span>
      </div>
    </div>

    <!-- Options -->
    <div class="space-y-3 mb-4">
      <div>
        <div class="flex items-center justify-between mb-1.5">
          <label class="text-sm font-medium">{{ t('passwordGen.length') }}</label>
          <span class="text-sm text-muted-foreground">{{ length }}</span>
        </div>
        <input v-model="length" type="range" min="8" max="64" step="1"
               class="w-full h-2 rounded-full appearance-none bg-muted cursor-pointer accent-primary" />
      </div>

      <div class="grid grid-cols-2 gap-2">
        <label class="flex items-center gap-2 text-sm">
          <input type="checkbox" v-model="uppercase" class="rounded border-input" />
          A-Z
        </label>
        <label class="flex items-center gap-2 text-sm">
          <input type="checkbox" v-model="lowercase" class="rounded border-input" />
          a-z
        </label>
        <label class="flex items-center gap-2 text-sm">
          <input type="checkbox" v-model="numbers" class="rounded border-input" />
          0-9
        </label>
        <label class="flex items-center gap-2 text-sm">
          <input type="checkbox" v-model="symbols" class="rounded border-input" />
          !@#$
        </label>
      </div>

      <label class="flex items-center gap-2 text-sm">
        <input type="checkbox" v-model="excludeAmbiguous" class="rounded border-input" />
        {{ t('passwordGen.excludeAmbiguous') }}
      </label>
    </div>

    <!-- Actions -->
    <div class="flex gap-2">
      <button @click="generate"
              class="flex-1 flex items-center justify-center gap-1.5 py-2.5 rounded-lg border border-border text-sm hover:bg-muted transition-colors">
        <RefreshCw class="w-4 h-4" />
        {{ t('passwordGen.regenerate') }}
      </button>
      <button @click="usePassword"
              :disabled="!generated"
              class="flex-1 py-2.5 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50 transition-colors">
        {{ t('passwordGen.usePassword') }}
      </button>
    </div>
  </div>
</template>
