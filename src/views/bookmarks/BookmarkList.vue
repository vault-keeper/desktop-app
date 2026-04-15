<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import type { Bookmark, BookmarkGroup } from '../../types'
import { useWorkspaceStore } from '../../stores/workspace'
import { Plus, Search, Trash2, FolderPlus, ExternalLink, Globe, X } from 'lucide-vue-next'

const workspaceStore = useWorkspaceStore()
const router = useRouter()
const { t } = useI18n()
const bookmarks = ref<Bookmark[]>([])
const groups = ref<BookmarkGroup[]>([])
const loading = ref(true)
const searchQuery = ref('')
const selectedGroupId = ref<string | null>(null)
const showCreateModal = ref(false)
const showGroupModal = ref(false)
const editingGroup = ref<BookmarkGroup | null>(null)
const pendingDeleteId = ref<string | null>(null)
const pendingDeleteGroupId = ref<string | null>(null)

type TagEntry = { id: string; name: string; color: string | null }
const tagMap = ref<Record<string, TagEntry[]>>({})

const form = ref({
  title: '',
  url: '',
  description: '',
  favicon_url: '',
  group_id: undefined as string | undefined,
})

const faviconErrors = ref<Record<string, boolean>>({})
const fetchingMeta = ref(false)

const EMOJI_LIST = [
  '📁','📂','🗂️','📌','📍','🔖','🏷️','⭐','🌟','💼',
  '🏠','💻','📱','🎮','🎵','🎬','📚','🔬','🔧','💰',
  '🛒','✈️','🍔','❤️','🎯','🔑','🛡️','📊','🎓','🌐',
  '🔗','🧩','🚀','💡','🎨','📝','🗓️','🔔','⚙️','🏆',
]
const groupForm = ref({ name: '', icon: '📁' })

const filteredBookmarks = computed(() => {
  let result = bookmarks.value
  if (selectedGroupId.value) {
    result = result.filter(b => b.group_id === selectedGroupId.value)
  }
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(b =>
      b.title.toLowerCase().includes(q) ||
      b.url.toLowerCase().includes(q) ||
      (b.description?.toLowerCase().includes(q))
    )
  }
  return result
})

async function loadData() {
  loading.value = true
  try {
    const wsId = workspaceStore.currentWorkspace?.id
    if (!wsId) return
    ;[bookmarks.value, groups.value] = await Promise.all([
      invoke<Bookmark[]>('list_bookmarks', { groupId: selectedGroupId.value }),
      invoke<BookmarkGroup[]>('list_bookmark_groups'),
    ])
    const entries = await invoke<{ entity_id: string; id: string; name: string; color: string | null }[]>(
      'list_all_entity_tags', { entityType: 'bookmark' }
    )
    const m: Record<string, TagEntry[]> = {}
    for (const e of entries) {
      if (!m[e.entity_id]) m[e.entity_id] = []
      m[e.entity_id].push({ id: e.id, name: e.name, color: e.color })
    }
    tagMap.value = m
  } catch (e) {
    console.error('Failed to load bookmarks:', e)
  } finally {
    loading.value = false
  }
}

function openCreateModal() {
  form.value = { title: '', url: '', description: '', favicon_url: '', group_id: selectedGroupId.value || undefined }
  showCreateModal.value = true
}

async function createBookmark() {
  if (!form.value.title || !form.value.url) return
  try {
    await invoke('create_bookmark', { data: {
      title: form.value.title,
      url: form.value.url,
      description: form.value.description || undefined,
      favicon_url: form.value.favicon_url || undefined,
      group_id: form.value.group_id,
    }})
    showCreateModal.value = false
    form.value = { title: '', url: '', description: '', favicon_url: '', group_id: undefined }
    await loadData()
  } catch (e) {
    console.error('Failed to create bookmark:', e)
  }
}

async function onUrlBlur() {
  const url = form.value.url.trim()
  if (!url || !url.startsWith('http')) return
  fetchingMeta.value = true
  try {
    const meta = await invoke<{ title: string; description: string; favicon_url: string }>(
      'fetch_url_metadata', { url }
    )
    if (!form.value.title && meta.title) form.value.title = meta.title
    if (!form.value.description && meta.description) form.value.description = meta.description
    if (meta.favicon_url) form.value.favicon_url = meta.favicon_url
  } catch {
    // silently ignore fetch errors
  } finally {
    fetchingMeta.value = false
  }
}

async function deleteBookmark(id: string, e: Event) {
  e.stopPropagation()
  pendingDeleteId.value = id
}

async function confirmDeleteBookmark() {
  if (!pendingDeleteId.value) return
  const id = pendingDeleteId.value
  pendingDeleteId.value = null
  await invoke('delete_bookmark', { id })
  await loadData()
}

async function createGroup() {
  if (!groupForm.value.name.trim()) return
  try {
    if (editingGroup.value) {
      await invoke('update_bookmark_group', {
        id: editingGroup.value.id,
        data: { name: groupForm.value.name, icon: groupForm.value.icon }
      })
    } else {
      await invoke('create_bookmark_group', { data: groupForm.value })
    }
    showGroupModal.value = false
    editingGroup.value = null
    groupForm.value = { name: '', icon: '📁' }
    await loadData()
  } catch (e) {
    console.error('Failed to save group:', e)
  }
}

function openEditGroup(g: BookmarkGroup) {
  editingGroup.value = g
  groupForm.value = { name: g.name, icon: g.icon ?? '📁' }
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
    await invoke('delete_bookmark_group', { id })
    if (selectedGroupId.value === id) selectedGroupId.value = null
    await loadData()
  } catch (e) {
    console.error('Failed to delete group:', e)
  }
}

function formatUrl(url: string) {
  try {
    const u = new URL(url)
    return u.hostname
  } catch {
    return url
  }
}

loadData()
</script>

<template>
  <div class="max-w-5xl mx-auto">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold">{{ t('bookmarks.title') }}</h1>
      <div class="flex items-center gap-2">
        <button @click="showGroupModal = true" class="flex items-center gap-1.5 px-3 py-2 rounded-lg border border-border text-sm hover:bg-muted transition-colors">
          <FolderPlus class="w-4 h-4" />
          {{ t('common.newGroup') }}
        </button>
        <button @click="openCreateModal()" class="flex items-center gap-1.5 px-3 py-2 rounded-lg bg-primary text-primary-foreground text-sm hover:bg-primary/90 transition-colors">
          <Plus class="w-4 h-4" />
          {{ t('bookmarks.addBtn') }}
        </button>
      </div>
    </div>

    <!-- Search -->
    <div class="mb-4">
      <div class="relative flex-1 max-w-md">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
        <input
          v-model="searchQuery"
          :placeholder="t('bookmarks.searchPlaceholder')"
          class="w-full pl-10 pr-4 py-2 rounded-lg border border-input bg-background text-sm
                 placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
        />
      </div>
    </div>

    <!-- Groups pills -->
    <div v-if="groups.length > 0" class="flex flex-wrap gap-2 mb-6">
      <button @click="selectedGroupId = null; loadData()"
              class="px-3 py-1.5 rounded-full text-sm border transition-colors"
              :class="!selectedGroupId ? 'bg-primary text-primary-foreground border-primary' : 'border-border hover:bg-muted'">
        {{ t('bookmarks.all') }}
      </button>
      <div v-for="g in groups" :key="g.id"
           class="inline-flex items-center gap-1 px-3 py-1.5 rounded-full text-sm border cursor-pointer transition-colors select-none"
           :class="selectedGroupId === g.id ? 'bg-primary text-primary-foreground border-primary' : 'border-border hover:bg-muted'"
           @click="selectedGroupId = g.id; loadData()" @dblclick="openEditGroup(g)" :title="t('common.dblClickEdit')">
        <span>{{ g.icon }}</span>
        <span>{{ g.name }}</span>
        <button v-if="selectedGroupId === g.id && filteredBookmarks.length === 0"
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

    <!-- Empty state -->
    <div v-else-if="filteredBookmarks.length === 0" class="text-center py-16 text-muted-foreground">
      <Globe class="w-12 h-12 mx-auto mb-4 opacity-50" />
      <p class="text-lg font-medium">{{ t('bookmarks.empty') }}</p>
      <p class="text-sm mt-1">{{ t('bookmarks.emptyPrompt') }}</p>
    </div>

    <!-- Bookmark list -->
    <div v-else class="space-y-2">
      <div
        v-for="bk in filteredBookmarks"
        :key="bk.id"
        @click="router.push({ name: 'bookmark-detail', params: { id: bk.id } })"
        class="group bg-card rounded-xl border border-border p-4 cursor-pointer
               hover:border-primary/50 hover:shadow-sm transition-all"
      >
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-lg bg-muted flex items-center justify-center text-lg shrink-0 overflow-hidden">
            <img
              v-if="bk.favicon_url && !faviconErrors[bk.id]"
              :src="bk.favicon_url"
              class="w-6 h-6 object-contain"
              @error="faviconErrors[bk.id] = true"
            />
            <span v-else class="select-none">🔗</span>
          </div>
          <div class="flex-1 min-w-0">
            <h3 class="font-medium text-sm truncate">{{ bk.title }}</h3>
            <p class="text-xs text-muted-foreground truncate">{{ formatUrl(bk.url) }}</p>
            <!-- Tags -->
            <div v-if="tagMap[bk.id]?.length" class="flex flex-wrap gap-1 mt-1.5">
              <span v-for="tag in tagMap[bk.id]" :key="tag.id"
                    class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium text-white leading-none"
                    :style="{ backgroundColor: tag.color ?? '#6366f1' }">
                #{{ tag.name }}
              </span>
            </div>
          </div>
          <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              @click.stop="openUrl(bk.url)"
              class="p-1.5 rounded-md hover:bg-muted"
              :title="t('common.openInBrowser')"
            >
              <ExternalLink class="w-4 h-4 text-muted-foreground" />
            </button>
            <button
              @click.stop="deleteBookmark(bk.id, $event)"
              class="p-1.5 rounded-md hover:bg-destructive/10 hover:text-destructive"
              :title="t('common.delete')"
            >
              <Trash2 class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Create Modal -->
    <div v-if="showCreateModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showCreateModal = false">
      <div class="bg-card rounded-xl border border-border p-6 w-full max-w-md shadow-xl">
        <h2 class="text-lg font-semibold mb-4">{{ t('bookmarks.addBtn') }}</h2>
        <div class="space-y-4">
          <div>
            <label class="text-sm font-medium mb-1.5 block">{{ t('bookmarks.titleLabel') }}</label>
            <input v-model="form.title" :placeholder="t('bookmarks.titlePlaceholder')"
                   class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                          focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" autofocus />
          </div>
          <div>
            <label class="text-sm font-medium mb-1.5 block">{{ t('common.url') }}</label>
            <div class="relative">
              <input v-model="form.url" :placeholder="t('bookmarks.urlPlaceholder')"
                     @blur="onUrlBlur"
                     class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                            focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
              <span v-if="fetchingMeta"
                    class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-muted-foreground animate-pulse">
                {{ t('bookmarks.fetching') }}
              </span>
            </div>
          </div>
          <div>
            <label class="text-sm font-medium mb-1.5 block">{{ t('bookmarks.descLabel') }}</label>
            <textarea v-model="form.description" rows="2" :placeholder="t('bookmarks.descPlaceholder')"
                      class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                             focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring resize-none" />
          </div>
          <div v-if="groups.length > 0">
            <label class="text-sm font-medium mb-1.5 block">{{ t('common.group') }}</label>
            <select v-model="form.group_id"
                    class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                           focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
              <option :value="undefined">{{ t('common.noGroup') }}</option>
              <option v-for="g in groups" :key="g.id" :value="g.id">
                {{ g.icon ? g.icon + ' ' : '' }}{{ g.name }}
              </option>
            </select>
          </div>
          <div class="flex gap-2 pt-2">
            <button @click="createBookmark" :disabled="!form.title || !form.url"
                    class="flex-1 py-2.5 rounded-lg bg-primary text-primary-foreground text-sm font-medium
                           hover:bg-primary/90 disabled:opacity-50 transition-colors">
              {{ t('bookmarks.createBtn') }}
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
    <div v-if="showGroupModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
         @click.self="showGroupModal = false; editingGroup = null; groupForm = { name: '', icon: '📁' }">
      <div class="bg-card rounded-xl border border-border p-6 w-full max-w-sm shadow-xl">
        <h2 class="text-lg font-semibold mb-4">{{ editingGroup ? t('common.editGroup') : t('common.newGroup') }}</h2>
        <div class="space-y-4">
          <div>
            <label class="text-sm font-medium mb-1.5 block">{{ t('common.name') }}</label>
            <input v-model="groupForm.name" :placeholder="t('common.name')"
                   class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                          focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" autofocus />
          </div>
          <div>
            <label class="text-sm font-medium mb-1.5 block">{{ t('common.icon') }}</label>
            <div class="flex items-center gap-3 mb-2">
              <span class="text-3xl w-10 h-10 flex items-center justify-center rounded-lg bg-muted">{{ groupForm.icon }}</span>
            </div>
            <div class="grid grid-cols-10 gap-1 p-2 bg-muted/50 rounded-lg max-h-32 overflow-y-auto">
              <button
                v-for="emoji in EMOJI_LIST" :key="emoji"
                type="button"
                @click="groupForm.icon = emoji"
                class="w-7 h-7 flex items-center justify-center rounded text-base hover:bg-background transition-colors"
                :class="groupForm.icon === emoji ? 'bg-background shadow-sm ring-1 ring-primary' : ''"
              >{{ emoji }}</button>
            </div>
          </div>
          <div class="flex gap-2 pt-2">
            <button @click="createGroup" :disabled="!groupForm.name.trim()"
                    class="flex-1 py-2.5 rounded-lg bg-primary text-primary-foreground text-sm font-medium
                           hover:bg-primary/90 disabled:opacity-50 transition-colors">
              {{ editingGroup ? t('common.save') : t('common.create') }}
            </button>
            <button @click="showGroupModal = false; editingGroup = null; groupForm = { name: '', icon: '📁' }"
                    class="px-4 py-2.5 text-sm text-muted-foreground hover:text-foreground transition-colors">
              {{ t('common.cancel') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Delete bookmark modal -->
    <div v-if="pendingDeleteId"
         class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
         @click.self="pendingDeleteId = null">
      <div class="w-full max-w-sm rounded-xl border border-border bg-card p-6 shadow-xl">
        <h2 class="text-lg font-semibold mb-1">{{ t('bookmarks.deleteTitle') }}</h2>
        <p class="text-sm text-muted-foreground mb-1">
          {{ t('bookmarks.deleteConfirm', { name: bookmarks.find(b => b.id === pendingDeleteId)?.title }) }}
        </p>
        <p class="text-sm text-destructive mb-5">{{ t('common.irreversible') }}</p>
        <div class="flex gap-2">
          <button @click="confirmDeleteBookmark"
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
          {{ t('bookmarks.deleteConfirm', { name: groups.find(g => g.id === pendingDeleteGroupId)?.name }) }}
        </p>
        <p class="text-sm text-destructive mb-5">{{ t('bookmarks.groupItemsKept') }}</p>
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
