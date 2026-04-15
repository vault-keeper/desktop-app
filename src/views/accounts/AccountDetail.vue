<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useI18n } from 'vue-i18n'
import type { Account, AccountGroup, Tag } from '../../types'
import {
  ArrowLeft, Save, Trash2, Eye, EyeOff, Copy, Check,
  Star, RefreshCw, ExternalLink
} from 'lucide-vue-next'
import TagInput from '../../components/TagInput.vue'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()

const account = ref<Account | null>(null)
const groups = ref<AccountGroup[]>([])
const loading = ref(true)
const saving = ref(false)
const showPassword = ref(false)
const showDeleteConfirm = ref(false)
const copiedField = ref<string | null>(null)
const error = ref('')
const accountTags = ref<Tag[]>([])

const editForm = ref({ title: '', url: '', username: '', password: '', notes: '', group_id: '' })

const hasChanges = computed(() => {
  if (!account.value) return false
  return (
    editForm.value.title    !== account.value.title ||
    editForm.value.url      !== (account.value.url      ?? '') ||
    editForm.value.username !== (account.value.username ?? '') ||
    editForm.value.password !== account.value.password ||
    editForm.value.notes    !== (account.value.notes    ?? '') ||
    editForm.value.group_id !== (account.value.group_id ?? '')
  )
})

async function loadAccount() {
  loading.value = true
  error.value = ''
  try {
    ;[account.value, groups.value] = await Promise.all([
      invoke<Account>('get_account', { id: route.params.id as string }),
      invoke<AccountGroup[]>('list_account_groups'),
    ])
    editForm.value = {
      title:    account.value.title,
      url:      account.value.url      ?? '',
      username: account.value.username ?? '',
      password: account.value.password,
      notes:    account.value.notes    ?? '',
      group_id: account.value.group_id ?? '',
    }
    accountTags.value = await invoke<Tag[]>('get_entity_tags', {
      entityType: 'account', entityId: route.params.id as string
    })
  } catch (e: any) { error.value = String(e) }
  finally { loading.value = false }
}

async function saveAccount() {
  if (!account.value || !hasChanges.value) return
  saving.value = true
  try {
    await invoke('update_account', {
      id: account.value.id,
      data: {
        title:    editForm.value.title,
        url:      editForm.value.url      || undefined,
        username: editForm.value.username || undefined,
        password: editForm.value.password || undefined,
        notes:    editForm.value.notes    || undefined,
        group_id: editForm.value.group_id || null,
      }
    })
    account.value = { ...account.value, ...editForm.value, group_id: editForm.value.group_id || null }
  } catch (e: any) { error.value = String(e) }
  finally { saving.value = false }
}

async function toggleFavorite() {
  if (!account.value) return
  const newVal = account.value.favorite ? 0 : 1
  await invoke('update_account', { id: account.value.id, data: { favorite: newVal } })
  account.value = { ...account.value, favorite: newVal }
}

async function deleteAccount() {
  if (!account.value) return
  await invoke('delete_account', { id: account.value.id })
  router.back()
}

async function generatePassword() {
  const pwd = await invoke<string>('generate_password', { length: 16 })
  editForm.value.password = pwd
  showPassword.value = true
}

function formatDate(iso: string) {
  const d = new Date(iso)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth()+1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

async function copyField(text: string, field: string) {
  try {
    await navigator.clipboard.writeText(text)
    copiedField.value = field
    setTimeout(() => { copiedField.value = null }, 1500)
  } catch {}
}

onMounted(loadAccount)
</script>

<template>
  <div class="max-w-2xl mx-auto">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div class="flex items-center gap-3">
        <button @click="router.back()"
                class="inline-flex items-center justify-center rounded-md text-sm font-medium
                       h-9 w-9 hover:bg-accent hover:text-accent-foreground transition-colors">
          <ArrowLeft class="w-4 h-4" />
        </button>
        <h1 class="text-xl font-semibold tracking-tight">{{ account?.title || t('accounts.detailTitle') }}</h1>
      </div>
      <div class="flex items-center gap-1.5">
        <button @click="toggleFavorite"
                class="inline-flex items-center justify-center rounded-md h-9 w-9 transition-colors
                       hover:bg-accent"
                :class="account?.favorite ? 'text-yellow-500' : 'text-muted-foreground hover:text-foreground'">
          <Star class="w-4 h-4" :fill="account?.favorite ? 'currentColor' : 'none'" />
        </button>
        <button @click="showDeleteConfirm = true"
                class="inline-flex items-center justify-center rounded-md h-9 w-9 text-muted-foreground
                       hover:bg-destructive/10 hover:text-destructive transition-colors">
          <Trash2 class="w-4 h-4" />
        </button>
        <button @click="saveAccount" :disabled="!hasChanges || saving"
                class="inline-flex items-center gap-1.5 rounded-md bg-primary px-3 h-9 text-sm
                       font-medium text-primary-foreground shadow hover:bg-primary/90
                       disabled:pointer-events-none disabled:opacity-50 transition-colors">
          <Save class="w-4 h-4" />
          {{ saving ? t('common.saving') : t('common.save') }}
        </button>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="rounded-xl border border-border bg-card p-6 space-y-4">
      <div v-for="i in 5" :key="i" class="h-10 rounded-md bg-muted animate-pulse" />
    </div>

    <!-- Error -->
    <div v-else-if="error"
         class="rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
      {{ error }}
    </div>

    <!-- Form -->
    <div v-else class="rounded-xl border border-border bg-card shadow-sm">
      <div class="p-6 space-y-5">

        <!-- Name -->
        <div class="space-y-2">
          <label class="text-sm font-medium leading-none">{{ t('accounts.siteLabel') }}</label>
          <input v-model="editForm.title" :placeholder="t('accounts.namePlaceholder')"
                 class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                        placeholder:text-muted-foreground focus-visible:outline-none
                        focus-visible:ring-1 focus-visible:ring-ring transition-colors" />
        </div>

        <!-- URL -->
        <div class="space-y-2">
          <label class="text-sm font-medium leading-none">{{ t('common.url') }}</label>
          <div class="flex gap-2">
            <input v-model="editForm.url" :placeholder="t('accounts.urlPlaceholder')"
                   class="flex h-10 flex-1 rounded-md border border-input bg-background px-3 py-2 text-sm
                          placeholder:text-muted-foreground focus-visible:outline-none
                          focus-visible:ring-1 focus-visible:ring-ring transition-colors" />
            <button v-if="editForm.url" @click="copyField(editForm.url, 'url')"
                    class="inline-flex items-center justify-center h-10 w-10 rounded-md border border-input
                           bg-background hover:bg-accent text-muted-foreground hover:text-foreground transition-colors">
              <Check v-if="copiedField === 'url'" class="w-4 h-4 text-green-500" />
              <Copy v-else class="w-4 h-4" />
            </button>
            <button v-if="editForm.url" @click="openUrl(editForm.url)"
                    class="inline-flex items-center justify-center h-10 w-10 rounded-md border border-input
                           bg-background hover:bg-accent text-muted-foreground hover:text-foreground transition-colors"
                    :title="t('common.openInBrowser')">
              <ExternalLink class="w-4 h-4" />
            </button>
          </div>
        </div>

        <!-- Username -->
        <div class="space-y-2">
          <label class="text-sm font-medium leading-none">{{ t('accounts.usernameLabel') }}</label>
          <div class="flex gap-2">
            <input v-model="editForm.username" :placeholder="t('accounts.usernamePlaceholder')"
                   class="flex h-10 flex-1 rounded-md border border-input bg-background px-3 py-2 text-sm
                          placeholder:text-muted-foreground focus-visible:outline-none
                          focus-visible:ring-1 focus-visible:ring-ring transition-colors" />
            <button v-if="editForm.username" @click="copyField(editForm.username, 'username')"
                    class="inline-flex items-center justify-center h-10 w-10 rounded-md border border-input
                           bg-background hover:bg-accent text-muted-foreground hover:text-foreground transition-colors">
              <Check v-if="copiedField === 'username'" class="w-4 h-4 text-green-500" />
              <Copy v-else class="w-4 h-4" />
            </button>
          </div>
        </div>

        <!-- Password -->
        <div class="space-y-2">
          <label class="text-sm font-medium leading-none">{{ t('accounts.passwordLabel') }}</label>
          <div class="flex gap-2">
            <div class="relative flex-1">
              <input v-model="editForm.password" :type="showPassword ? 'text' : 'password'"
                     :placeholder="t('accounts.passwordLabel')"
                     class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 pr-10
                            text-sm font-mono placeholder:text-muted-foreground focus-visible:outline-none
                            focus-visible:ring-1 focus-visible:ring-ring transition-colors" />
              <button @click="showPassword = !showPassword"
                      class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground">
                <Eye v-if="!showPassword" class="w-4 h-4" />
                <EyeOff v-else class="w-4 h-4" />
              </button>
            </div>
            <button v-if="editForm.password" @click="copyField(editForm.password, 'password')"
                    class="inline-flex items-center justify-center h-10 w-10 rounded-md border border-input
                           bg-background hover:bg-accent text-muted-foreground hover:text-foreground transition-colors">
              <Check v-if="copiedField === 'password'" class="w-4 h-4 text-green-500" />
              <Copy v-else class="w-4 h-4" />
            </button>
            <button @click="generatePassword" :title="t('accounts.createForm.generateTitle')"
                    class="inline-flex items-center justify-center h-10 w-10 rounded-md border border-input
                           bg-background hover:bg-accent text-muted-foreground hover:text-foreground transition-colors">
              <RefreshCw class="w-4 h-4" />
            </button>
          </div>
        </div>

        <!-- Notes -->
        <div class="space-y-2">
          <label class="text-sm font-medium leading-none">{{ t('accounts.notesLabel') }}</label>
          <textarea v-model="editForm.notes" :placeholder="t('accounts.notesPlaceholder')" rows="3"
                    class="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                           placeholder:text-muted-foreground focus-visible:outline-none
                           focus-visible:ring-1 focus-visible:ring-ring resize-none transition-colors" />
        </div>

        <!-- Group -->
        <div v-if="groups.length > 0" class="space-y-2">
          <label class="text-sm font-medium leading-none">{{ t('common.group') }}</label>
          <select v-model="editForm.group_id"
                  class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                         focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring transition-colors">
            <option value="">{{ t('common.noGroup') }}</option>
            <option v-for="g in groups" :key="g.id" :value="g.id">
              {{ g.icon ? g.icon + ' ' : '' }}{{ g.name }}
            </option>
          </select>
        </div>

        <!-- Tags -->
        <div class="space-y-2">
          <label class="text-sm font-medium leading-none">{{ t('common.tags') }}</label>
          <TagInput v-model="accountTags" entity-type="account" :entity-id="account?.id ?? ''" />
        </div>

      </div>

      <!-- Footer meta -->
      <div v-if="account" class="border-t border-border px-6 py-3">
        <p class="text-xs text-muted-foreground">
          {{ t('common.createdAt') }}{{ formatDate(account.created_at) }}
          · {{ t('common.updatedAt') }}{{ formatDate(account.updated_at) }}
        </p>
      </div>
    </div>

    <!-- Delete dialog -->
    <div v-if="showDeleteConfirm"
         class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
         @click.self="showDeleteConfirm = false">
      <div class="w-full max-w-sm rounded-xl border border-border bg-card p-6 shadow-xl">
        <h2 class="text-lg font-semibold mb-1">{{ t('accounts.deleteTitle') }}</h2>
        <p class="text-sm text-muted-foreground mb-1">
          {{ t('accounts.deleteConfirm', { name: account?.title }) }}
        </p>
        <p class="text-sm text-destructive mb-5">{{ t('common.irreversible') }}</p>
        <div class="flex gap-2">
          <button @click="deleteAccount"
                  class="flex-1 h-9 rounded-md bg-destructive text-destructive-foreground text-sm font-medium
                         hover:bg-destructive/90 transition-colors">{{ t('common.confirmDelete') }}</button>
          <button @click="showDeleteConfirm = false"
                  class="flex-1 h-9 rounded-md border border-input bg-background text-sm
                         hover:bg-accent transition-colors">{{ t('common.cancel') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>
