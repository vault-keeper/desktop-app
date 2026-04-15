<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import type { Account, CreateAccount, Tag } from '../../types'
import { Plus, Search, Trash2, FolderPlus, Copy, Check, Eye, EyeOff, Star, RefreshCw, ExternalLink, X } from 'lucide-vue-next'
import PasswordGenerator from '../../components/common/PasswordGenerator.vue'
import TagInput from '../../components/TagInput.vue'

const router = useRouter()
const { t } = useI18n()
const accounts = ref<Account[]>([])
const groups = ref<any[]>([])
const loading = ref(true)
const searchQuery = ref('')
const selectedGroupId = ref<string | null>(null)
const showCreateModal = ref(false)
const showGenModal = ref(false)
const showGroupModal = ref(false)
const editingGroup = ref<any | null>(null)  // null = create, non-null = edit
const showPassword = ref<Record<string, boolean>>({})
const copiedId = ref<string | null>(null)
const pendingDeleteId = ref<string | null>(null)
const pendingDeleteGroupId = ref<string | null>(null)

// tag map: accountId -> [{id, name, color}]
type TagEntry = { id: string; name: string; color: string | null }
const tagMap = ref<Record<string, TagEntry[]>>({})

const form = ref<CreateAccount>({
  title: '', url: '', username: '', password: '', notes: '', group_id: undefined
})
const showFormPassword = ref(false)
const pendingTags = ref<Tag[]>([])

const EMOJI_LIST = [
  '🔑','🔐','🔒','🛡️','💼','🏦','💳','🧾','🌐','💻',
  '📱','🎮','📧','🔗','🏠','🏢','🎓','🏥','🚀','⭐',
  '🌟','💡','🎯','📊','🎨','🔧','⚙️','🏆','❤️','🎵',
  '📁','📂','🗂️','📌','📍','🔖','🏷️','📝','🗓️','🔔',
]
const groupForm = ref({ name: '', icon: '🔑' })

async function loadData() {
  loading.value = true
  try {
    ;[accounts.value, groups.value] = await Promise.all([
      invoke<Account[]>('list_accounts', { groupId: selectedGroupId.value }),
      invoke<any[]>('list_account_groups'),
    ])
    // Batch-load tags
    const entries = await invoke<{ entity_id: string; id: string; name: string; color: string | null }[]>(
      'list_all_entity_tags', { entityType: 'account' }
    )
    const m: Record<string, TagEntry[]> = {}
    for (const e of entries) {
      if (!m[e.entity_id]) m[e.entity_id] = []
      m[e.entity_id].push({ id: e.id, name: e.name, color: e.color })
    }
    tagMap.value = m
  } catch (e) {
    console.error('Failed to load accounts:', e)
  } finally {
    loading.value = false
  }
}

async function createGroup() {
  if (!groupForm.value.name.trim()) return
  try {
    if (editingGroup.value) {
      await invoke('update_account_group', {
        id: editingGroup.value.id,
        data: { name: groupForm.value.name, icon: groupForm.value.icon }
      })
    } else {
      await invoke('create_account_group', { data: { name: groupForm.value.name, icon: groupForm.value.icon } })
    }
    showGroupModal.value = false
    editingGroup.value = null
    groupForm.value = { name: '', icon: '🔑' }
    await loadData()
  } catch (e) {
    console.error('Failed to save group:', e)
  }
}

function openEditGroup(g: any) {
  editingGroup.value = g
  groupForm.value = { name: g.name, icon: g.icon ?? '🔑' }
  showGroupModal.value = true
}

async function deleteGroup(id: string) {
  pendingDeleteGroupId.value = id
}

async function confirmDeleteGroup() {
  if (!pendingDeleteGroupId.value) return
  const id = pendingDeleteGroupId.value
  pendingDeleteGroupId.value = null
  try {
    await invoke('delete_account_group', { id })
    if (selectedGroupId.value === id) selectedGroupId.value = null
    await loadData()
  } catch (e) {
    console.error('Failed to delete group:', e)
  }
}

function openCreateModal() {
  form.value = { title: '', url: '', username: '', password: '', notes: '', group_id: selectedGroupId.value || undefined }
  showCreateModal.value = true
}

async function createAccount() {
  if (!form.value.title || !form.value.password) return
  try {
    const created = await invoke<Account>('create_account', {
      data: {
        title: form.value.title,
        url: form.value.url || undefined,
        username: form.value.username || undefined,
        password: form.value.password,
        notes: form.value.notes || undefined,
        group_id: form.value.group_id || undefined,
      }
    })
    for (const tag of pendingTags.value) {
      await invoke('tag_entity', { tagId: tag.id, entityType: 'account', entityId: created.id }).catch(() => {})
    }
    showCreateModal.value = false
    form.value = { title: '', url: '', username: '', password: '', notes: '', group_id: undefined }
    showFormPassword.value = false
    pendingTags.value = []
    await loadData()
  } catch (e) {
    console.error('Failed to create account:', e)
  }
}

async function deleteAccount(id: string, e: Event) {
  e.stopPropagation()
  pendingDeleteId.value = id
}

async function confirmDeleteAccount() {
  if (!pendingDeleteId.value) return
  const id = pendingDeleteId.value
  pendingDeleteId.value = null
  await invoke('delete_account', { id })
  await loadData()
}

function togglePassword(id: string) {
  showPassword.value[id] = !showPassword.value[id]
}

async function copyToClipboard(text: string, id: string) {
  try {
    await navigator.clipboard.writeText(text)
    copiedId.value = id
    setTimeout(() => { copiedId.value = null }, 1500)
  } catch {}
}

function maskPassword(pwd: string): string {
  return '•'.repeat(Math.min(pwd.length, 16))
}

function openDetail(id: string) {
  router.push({ name: 'account-detail', params: { id } })
}

function formatUrl(url: string) {
  try { return new URL(url).hostname } catch { return url }
}

async function generatePassword(length: number) {
  const pwd = await invoke<string>('generate_password', { length })
  form.value.password = pwd
  showFormPassword.value = true
}

function selectGroup(id: string | null) {
  selectedGroupId.value = id
  loadData()
}

loadData()
</script>

<template>
  <div class="max-w-5xl mx-auto">
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold">{{ t('accounts.title') }}</h1>
      <div class="flex items-center gap-2">
        <button @click="showGroupModal = true"
                class="flex items-center gap-1.5 px-3 py-2 rounded-lg border border-border text-sm hover:bg-muted transition-colors">
          <FolderPlus class="w-4 h-4" />
          {{ t('common.newGroup') }}
        </button>
        <button @click="showGenModal = true"
                class="flex items-center gap-1.5 px-3 py-2 rounded-lg border border-border text-sm hover:bg-muted transition-colors">
          <RefreshCw class="w-4 h-4" />
          {{ t('accounts.createForm.generateTitle') }}
        </button>
        <button @click="openCreateModal()"
                class="flex items-center gap-1.5 px-3 py-2 rounded-lg bg-primary text-primary-foreground text-sm hover:bg-primary/90 transition-colors">
          <Plus class="w-4 h-4" />
          {{ t('accounts.addBtn') }}
        </button>
      </div>
    </div>

    <!-- Search -->
    <div class="mb-4">
      <div class="relative max-w-md">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
        <input v-model="searchQuery" :placeholder="t('accounts.searchPlaceholder')"
               class="w-full pl-10 pr-4 py-2 rounded-lg border border-input bg-background text-sm
                      placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
      </div>
    </div>

    <!-- Group pills -->
    <div v-if="groups.length > 0" class="flex flex-wrap gap-2 mb-6">
      <button @click="selectGroup(null)"
              class="px-3 py-1.5 rounded-full text-sm border transition-colors"
              :class="!selectedGroupId ? 'bg-primary text-primary-foreground border-primary' : 'border-border hover:bg-muted'">
        {{ t('accounts.all') }}
      </button>
      <div v-for="g in groups" :key="g.id"
           class="inline-flex items-center gap-1 px-3 py-1.5 rounded-full text-sm border cursor-pointer transition-colors select-none"
           :class="selectedGroupId === g.id ? 'bg-primary text-primary-foreground border-primary' : 'border-border hover:bg-muted'"
           @click="selectGroup(g.id)" @dblclick="openEditGroup(g)" :title="t('common.dblClickEdit')">
        <span v-if="g.icon">{{ g.icon }}</span>
        <span>{{ g.name }}</span>
        <button v-if="selectedGroupId === g.id && accounts.length === 0"
                @click.stop="deleteGroup(g.id)"
                class="ml-0.5 -mr-1 flex items-center justify-center w-5 h-5 rounded-full bg-destructive text-destructive-foreground hover:bg-destructive/80 transition-colors shrink-0"
                :title="t('common.deleteEmptyGroup')">
          <X class="w-3 h-3" />
        </button>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="space-y-3">
      <div v-for="i in 5" :key="i" class="h-16 bg-muted/50 rounded-lg animate-pulse" />
    </div>

    <div v-else-if="accounts.length === 0" class="text-center py-16 text-muted-foreground">
      <p class="text-lg font-medium">{{ t('accounts.empty') }}</p>
      <p class="text-sm mt-1">{{ t('accounts.emptyPrompt') }}</p>
    </div>

    <div v-else class="space-y-2">
      <div v-for="acc in accounts" :key="acc.id"
           @click="openDetail(acc.id)"
           class="group bg-card rounded-xl border border-border p-4 cursor-pointer
                  hover:border-primary/50 hover:shadow-sm transition-all">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg bg-muted flex items-center justify-center text-lg shrink-0">🔑</div>
          <div class="flex-1 min-w-0">
            <h3 class="font-medium text-sm">{{ acc.title }}</h3>
            <p class="text-xs text-muted-foreground">
              {{ acc.username || t('accounts.noUsername') }} ·
              {{ showPassword[acc.id] ? acc.password : maskPassword(acc.password) }}
            </p>
            <p v-if="acc.url" class="text-xs text-muted-foreground/70 truncate">{{ formatUrl(acc.url) }}</p>
            <!-- Tags -->
            <div v-if="tagMap[acc.id]?.length" class="flex flex-wrap gap-1 mt-1.5">
              <span v-for="tag in tagMap[acc.id]" :key="tag.id"
                    class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium text-white leading-none"
                    :style="{ backgroundColor: tag.color ?? '#6366f1' }">
                #{{ tag.name }}
              </span>
            </div>
          </div>
          <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
            <button v-if="acc.url" @click.stop="openUrl(acc.url)" class="p-1.5 rounded-md hover:bg-muted" :title="t('common.openInBrowser')">
              <ExternalLink class="w-4 h-4 text-muted-foreground" />
            </button>
            <button @click.stop="togglePassword(acc.id)" class="p-1.5 rounded-md hover:bg-muted" :title="t('accounts.showPassword')">
              <Eye v-if="!showPassword[acc.id]" class="w-4 h-4 text-muted-foreground" />
              <EyeOff v-else class="w-4 h-4 text-muted-foreground" />
            </button>
            <button @click.stop="copyToClipboard(acc.password, acc.id)" class="p-1.5 rounded-md hover:bg-muted" :title="t('accounts.copyPassword')">
              <Check v-if="copiedId === acc.id" class="w-4 h-4 text-green-500" />
              <Copy v-else class="w-4 h-4 text-muted-foreground" />
            </button>
            <button @click.stop="deleteAccount(acc.id, $event)" class="p-1.5 rounded-md hover:bg-destructive/10 hover:text-destructive" :title="t('common.delete')">
              <Trash2 class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Create Modal -->
    <div v-if="showCreateModal"
         class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
         @click.self="showCreateModal = false; pendingTags = []">
      <div class="bg-card rounded-xl border border-border p-6 w-full max-w-md shadow-xl">
        <h2 class="text-lg font-semibold mb-4">{{ t('accounts.createForm.title') }}</h2>
        <div class="space-y-3">
          <input v-model="form.title" :placeholder="t('accounts.createForm.sitePlaceholder')"
                 class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                 autofocus />
          <input v-model="form.url" :placeholder="t('accounts.createForm.urlPlaceholder')"
                 class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
          <input v-model="form.username" :placeholder="t('accounts.createForm.usernamePlaceholder')"
                 class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
          <div class="flex gap-2">
            <div class="relative flex-1">
              <input v-model="form.password" :placeholder="t('accounts.createForm.passwordPlaceholder')"
                     :type="showFormPassword ? 'text' : 'password'"
                     class="w-full px-4 py-2.5 pr-10 rounded-lg border border-input bg-background text-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
              <button type="button" @click="showFormPassword = !showFormPassword"
                      class="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground">
                <Eye v-if="!showFormPassword" class="w-4 h-4" />
                <EyeOff v-else class="w-4 h-4" />
              </button>
            </div>
            <button @click="generatePassword(16)"
                    class="px-3 rounded-lg border border-border hover:bg-muted transition-colors text-sm" :title="t('accounts.createForm.generateTitle')">
              <RefreshCw class="w-4 h-4" />
            </button>
          </div>
          <select v-if="groups.length > 0" v-model="form.group_id"
                  class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
            <option :value="undefined">{{ t('common.noGroup') }}</option>
            <option v-for="g in groups" :key="g.id" :value="g.id">{{ g.icon ? g.icon + ' ' : '' }}{{ g.name }}</option>
          </select>
          <textarea v-model="form.notes" :placeholder="t('accounts.createForm.notesPlaceholder')" rows="2"
                    class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring resize-none" />
          <div class="space-y-1.5">
            <label class="text-sm font-medium text-foreground">{{ t('common.tags') }}</label>
            <TagInput v-model="pendingTags" />
          </div>
          <div class="flex gap-2 pt-2">
            <button @click="createAccount" :disabled="!form.title || !form.password"
                    class="flex-1 py-2.5 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50 transition-colors">
              {{ t('accounts.createForm.createBtn') }}
            </button>
            <button @click="showCreateModal = false"
                    class="px-4 py-2.5 text-sm text-muted-foreground hover:text-foreground transition-colors">
              {{ t('common.cancel') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Create / Edit Group Modal -->
    <div v-if="showGroupModal"
         class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
         @click.self="showGroupModal = false; editingGroup = null; groupForm = { name: '', icon: '🔑' }">
      <div class="bg-card rounded-xl border border-border p-6 w-full max-w-sm shadow-xl">
        <h2 class="text-lg font-semibold mb-4">{{ editingGroup ? t('common.editGroup') : t('common.newGroup') }}</h2>
        <div class="space-y-4">
          <div>
            <label class="text-sm font-medium mb-1.5 block">{{ t('common.name') }}</label>
            <input v-model="groupForm.name" :placeholder="t('common.name')" autofocus
                   class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
          </div>
          <div>
            <label class="text-sm font-medium mb-1.5 block">{{ t('common.icon') }}</label>
            <div class="flex items-center gap-3 mb-2">
              <span class="text-3xl w-10 h-10 flex items-center justify-center rounded-lg bg-muted">{{ groupForm.icon }}</span>
            </div>
            <div class="grid grid-cols-10 gap-1 p-2 bg-muted/50 rounded-lg max-h-32 overflow-y-auto">
              <button v-for="emoji in EMOJI_LIST" :key="emoji"
                      type="button"
                      @click="groupForm.icon = emoji"
                      class="w-7 h-7 flex items-center justify-center rounded text-base hover:bg-background transition-colors"
                      :class="groupForm.icon === emoji ? 'bg-background shadow-sm ring-1 ring-primary' : ''">
                {{ emoji }}
              </button>
            </div>
          </div>
          <div class="flex gap-2 pt-2">
            <button @click="createGroup" :disabled="!groupForm.name.trim()"
                    class="flex-1 py-2.5 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50 transition-colors">
              {{ editingGroup ? t('common.save') : t('common.create') }}
            </button>
            <button @click="showGroupModal = false; editingGroup = null; groupForm = { name: '', icon: '🔑' }"
                    class="px-4 py-2.5 text-sm text-muted-foreground hover:text-foreground transition-colors">
              {{ t('common.cancel') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Password Generator Modal -->
    <div v-if="showGenModal"
         class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
         @click.self="showGenModal = false">
      <PasswordGenerator @close="showGenModal = false" @select="(p: string) => { form.password = p; showGenModal = false }" />
    </div>

    <!-- Delete account modal -->
    <div v-if="pendingDeleteId"
         class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
         @click.self="pendingDeleteId = null">
      <div class="w-full max-w-sm rounded-xl border border-border bg-card p-6 shadow-xl">
        <h2 class="text-lg font-semibold mb-1">{{ t('accounts.deleteTitle') }}</h2>
        <p class="text-sm text-muted-foreground mb-1">
          {{ t('accounts.deleteConfirm', { name: accounts.find(a => a.id === pendingDeleteId)?.title }) }}
        </p>
        <p class="text-sm text-destructive mb-5">{{ t('common.irreversible') }}</p>
        <div class="flex gap-2">
          <button @click="confirmDeleteAccount"
                  class="flex-1 h-9 rounded-md bg-destructive text-destructive-foreground text-sm font-medium hover:bg-destructive/90 transition-colors">
            {{ t('common.confirmDelete') }}
          </button>
          <button @click="pendingDeleteId = null"
                  class="flex-1 h-9 rounded-md border border-input bg-background text-sm hover:bg-accent transition-colors">
            {{ t('common.cancel') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Delete group modal -->
    <div v-if="pendingDeleteGroupId"
         class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
         @click.self="pendingDeleteGroupId = null">
      <div class="w-full max-w-sm rounded-xl border border-border bg-card p-6 shadow-xl">
        <h2 class="text-lg font-semibold mb-1">{{ t('common.deleteGroup') }}</h2>
        <p class="text-sm text-muted-foreground mb-1">
          {{ t('accounts.deleteConfirm', { name: groups.find(g => g.id === pendingDeleteGroupId)?.name }) }}
        </p>
        <p class="text-sm text-destructive mb-5">{{ t('accounts.groupItemsKept') }}</p>
        <div class="flex gap-2">
          <button @click="confirmDeleteGroup"
                  class="flex-1 h-9 rounded-md bg-destructive text-destructive-foreground text-sm font-medium hover:bg-destructive/90 transition-colors">
            {{ t('common.confirmDelete') }}
          </button>
          <button @click="pendingDeleteGroupId = null"
                  class="flex-1 h-9 rounded-md border border-input bg-background text-sm hover:bg-accent transition-colors">
            {{ t('common.cancel') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
