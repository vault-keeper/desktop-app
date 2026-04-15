<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import type { Note, NoteGroup, CreateNote } from '../../types'
import { useWorkspaceStore } from '../../stores/workspace'
import { Plus, Search, Trash2, FolderPlus, Lock, FileText, X } from 'lucide-vue-next'

const router = useRouter()
const workspaceStore = useWorkspaceStore()
const { t } = useI18n()
const notes = ref<Note[]>([])
const groups = ref<NoteGroup[]>([])
const loading = ref(true)
const searchQuery = ref('')
const selectedGroupId = ref<string | null>(null)
const showCreateModal = ref(false)
const showGroupModal = ref(false)
const editingGroup = ref<NoteGroup | null>(null)
const form = ref<CreateNote>({ title: '', content: '', group_id: undefined })
const groupForm = ref({ name: '', icon: '📝' })
const pendingDeleteId = ref<string | null>(null)
const pendingDeleteGroupId = ref<string | null>(null)

type TagEntry = { id: string; name: string; color: string | null }
const tagMap = ref<Record<string, TagEntry[]>>({})

const EMOJI_LIST = [
  '📝','📒','📓','📔','📕','📗','📘','📙','📚','🗒️',
  '📁','📂','🗂️','📌','📍','🔖','🏷️','⭐','🌟','💼',
  '🏠','💻','📱','🎮','🎵','🎬','🔬','🔧','💡','🎯',
  '🔑','🛡️','📊','🎓','🌐','🚀','🎨','⚙️','🏆','❤️',
]

const filteredNotes = ref<Note[]>([])

function filterNotes() {
  let result = notes.value
  if (selectedGroupId.value) {
    result = result.filter(n => n.group_id === selectedGroupId.value)
  }
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(n =>
      n.title.toLowerCase().includes(q) ||
      (!n.is_encrypted && n.content.toLowerCase().includes(q))
    )
  }
  filteredNotes.value = result
}

async function loadData() {
  loading.value = true
  try {
    ;[notes.value, groups.value] = await Promise.all([
      invoke<Note[]>('list_notes', { groupId: selectedGroupId.value }),
      invoke<NoteGroup[]>('list_note_groups'),
    ])
    filterNotes()
    const entries = await invoke<{ entity_id: string; id: string; name: string; color: string | null }[]>(
      'list_all_entity_tags', { entityType: 'note' }
    )
    const m: Record<string, TagEntry[]> = {}
    for (const e of entries) {
      if (!m[e.entity_id]) m[e.entity_id] = []
      m[e.entity_id].push({ id: e.id, name: e.name, color: e.color })
    }
    tagMap.value = m
  } catch (e) {
    console.error('Failed to load notes:', e)
  } finally {
    loading.value = false
  }
}

function openCreateModal() {
  form.value = { title: '', content: '', group_id: selectedGroupId.value || undefined }
  showCreateModal.value = true
}

async function createNote() {
  if (!form.value.title) return
  try {
    const note = await invoke<Note>('create_note', {
      data: {
        title: form.value.title,
        content: form.value.content,
        group_id: form.value.group_id || undefined,
      }
    })
    showCreateModal.value = false
    form.value = { title: '', content: '', group_id: undefined }
    router.push({ name: 'note-edit', params: { id: note.id } })
  } catch (e) {
    console.error('Failed to create note:', e)
  }
}

async function createGroup() {
  if (!groupForm.value.name.trim()) return
  try {
    if (editingGroup.value) {
      await invoke('update_note_group', {
        id: editingGroup.value.id,
        data: { name: groupForm.value.name, icon: groupForm.value.icon }
      })
    } else {
      await invoke('create_note_group', { data: groupForm.value })
    }
    showGroupModal.value = false
    editingGroup.value = null
    groupForm.value = { name: '', icon: '📝' }
    await loadData()
  } catch (e) {
    console.error('Failed to save group:', e)
  }
}

function openEditGroup(g: NoteGroup) {
  editingGroup.value = g
  groupForm.value = { name: g.name, icon: g.icon ?? '📝' }
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
    await invoke('delete_note_group', { id })
    if (selectedGroupId.value === id) selectedGroupId.value = null
    await loadData()
  } catch (e) {
    console.error('Failed to delete group:', e)
  }
}

async function deleteNote(id: string, e: Event) {
  e.stopPropagation()
  pendingDeleteId.value = id
}

async function confirmDeleteNote() {
  if (!pendingDeleteId.value) return
  const id = pendingDeleteId.value
  pendingDeleteId.value = null
  await invoke('delete_note', { id })
  await loadData()
}

function stripMarkdown(text: string): string {
  return text
    .replace(/#{1,6}\s+/g, '')
    .replace(/\*\*|__|~~|`/g, '')
    .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
    .replace(/^\s*[-*+]\s+/gm, '')
    .replace(/^\s*\d+\.\s+/gm, '')
    .replace(/\n+/g, ' ')
    .trim()
}

onMounted(loadData)
</script>

<template>
  <div class="max-w-5xl mx-auto">
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold">{{ t('notes.title') }}</h1>
      <div class="flex items-center gap-2">
        <button @click="showGroupModal = true" class="flex items-center gap-1.5 px-3 py-2 rounded-lg border border-border text-sm hover:bg-muted transition-colors">
          <FolderPlus class="w-4 h-4" />
          {{ t('common.newGroup') }}
        </button>
        <button @click="openCreateModal()" class="flex items-center gap-1.5 px-3 py-2 rounded-lg bg-primary text-primary-foreground text-sm hover:bg-primary/90 transition-colors">
          <Plus class="w-4 h-4" />
          {{ t('notes.newNote') }}
        </button>
      </div>
    </div>

    <div class="flex items-center gap-4 mb-4">
      <div class="relative flex-1 max-w-md">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
        <input v-model="searchQuery" @input="filterNotes" :placeholder="t('notes.searchPlaceholder')"
               class="w-full pl-10 pr-4 py-2 rounded-lg border border-input bg-background text-sm
                      placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
      </div>
    </div>

    <!-- Group pills -->
    <div v-if="groups.length > 0" class="flex flex-wrap gap-2 mb-6">
      <button @click="selectedGroupId = null; loadData()"
              class="px-3 py-1.5 rounded-full text-sm border transition-colors"
              :class="!selectedGroupId ? 'bg-primary text-primary-foreground border-primary' : 'border-border hover:bg-muted'">
        {{ t('notes.all') }}
      </button>
      <div v-for="g in groups" :key="g.id"
           class="inline-flex items-center gap-1 px-3 py-1.5 rounded-full text-sm border cursor-pointer transition-colors select-none"
           :class="selectedGroupId === g.id ? 'bg-primary text-primary-foreground border-primary' : 'border-border hover:bg-muted'"
           @click="selectedGroupId = g.id; loadData()" @dblclick="openEditGroup(g)" :title="t('common.dblClickEdit')">
        <span>{{ g.icon }}</span><span>{{ g.name }}</span>
        <button v-if="selectedGroupId === g.id && filteredNotes.length === 0"
                @click.stop="deleteGroup(g.id)"
                class="ml-0.5 -mr-1 flex items-center justify-center w-5 h-5 rounded-full bg-destructive text-destructive-foreground hover:bg-destructive/80 transition-colors shrink-0"
                :title="t('common.deleteEmptyGroup')">
          <X class="w-3 h-3" />
        </button>
      </div>
    </div>

    <div v-if="loading" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
      <div v-for="i in 6" :key="i" class="h-40 bg-muted/50 rounded-xl animate-pulse" />
    </div>

    <div v-else-if="filteredNotes.length === 0" class="text-center py-16 text-muted-foreground">
      <FileText class="w-12 h-12 mx-auto mb-4 opacity-50" />
      <p class="text-lg font-medium">{{ t('notes.empty') }}</p>
      <p class="text-sm mt-1">{{ t('notes.emptyPrompt') }}</p>
    </div>

    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
      <div
        v-for="note in filteredNotes"
        :key="note.id"
        @click="router.push({ name: 'note-detail', params: { id: note.id } })"
        class="group bg-card rounded-xl border border-border p-4 cursor-pointer
               hover:border-primary/50 hover:shadow-sm transition-all flex flex-col gap-2"
      >
        <div class="flex items-start justify-between">
          <h3 class="font-medium text-sm line-clamp-1 flex-1">{{ note.title }}</h3>
          <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity shrink-0 ml-2">
            <button @click.stop="deleteNote(note.id, $event)" class="p-1 rounded hover:bg-destructive/10 hover:text-destructive">
              <Trash2 class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
        <div v-if="tagMap[note.id]?.length" class="flex flex-wrap gap-1">
          <span v-for="tag in tagMap[note.id]" :key="tag.id"
                class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-semibold text-white"
                :style="{ backgroundColor: tag.color ?? '#6366f1' }">
            #{{ tag.name }}
          </span>
        </div>
        <!-- 加密笔记不显示内容摘要 -->
        <p v-if="!note.is_encrypted" class="text-xs text-muted-foreground line-clamp-2 flex-1">
          {{ stripMarkdown(note.content) }}
        </p>
        <div v-if="note.is_encrypted" class="flex items-center gap-1 text-xs text-amber-600">
          <Lock class="w-3 h-3" />
          {{ t('notes.encrypted') }}
        </div>
      </div>
    </div>

    <!-- Create Modal -->
    <div v-if="showCreateModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="showCreateModal = false">
      <div class="bg-card rounded-xl border border-border p-6 w-full max-w-md shadow-xl">
        <h2 class="text-lg font-semibold mb-4">{{ t('notes.createModal.title') }}</h2>
        <div class="space-y-3">
          <input v-model="form.title" :placeholder="t('notes.createModal.titlePlaceholder')" autofocus
                 class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                        focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
          <select v-if="groups.length > 0" v-model="form.group_id"
                  class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                         focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
            <option :value="undefined">{{ t('common.noGroup') }}</option>
            <option v-for="g in groups" :key="g.id" :value="g.id">{{ g.icon }} {{ g.name }}</option>
          </select>
          <div class="flex gap-2 pt-2">
            <button @click="createNote" :disabled="!form.title"
                    class="flex-1 py-2.5 rounded-lg bg-primary text-primary-foreground text-sm font-medium
                           hover:bg-primary/90 disabled:opacity-50 transition-colors">
              {{ t('notes.createModal.createBtn') }}
            </button>
            <button @click="showCreateModal = false" class="px-4 py-2.5 text-sm text-muted-foreground hover:text-foreground">
              {{ t('common.cancel') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Create / Edit Group Modal -->
    <div v-if="showGroupModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
         @click.self="showGroupModal = false; editingGroup = null; groupForm = { name: '', icon: '📝' }">
      <div class="bg-card rounded-xl border border-border p-6 w-full max-w-sm shadow-xl">
        <h2 class="text-lg font-semibold mb-4">{{ editingGroup ? t('common.editGroup') : t('common.newGroup') }}</h2>
        <div class="space-y-4">
          <div>
            <label class="text-sm font-medium mb-1.5 block">{{ t('common.name') }}</label>
            <input v-model="groupForm.name" :placeholder="t('notes.titlePlaceholder')" autofocus
                   class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm
                          focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
          </div>
          <div>
            <label class="text-sm font-medium mb-2 block">{{ t('common.icon') }}</label>
            <div class="flex items-center gap-3 mb-2">
              <span class="text-3xl w-10 h-10 flex items-center justify-center rounded-lg bg-muted">{{ groupForm.icon }}</span>
              <span class="text-sm text-muted-foreground">{{ t('notes.createModal.titlePlaceholder') }}</span>
            </div>
            <div class="grid grid-cols-10 gap-1 p-2 bg-muted/50 rounded-lg max-h-32 overflow-y-auto">
              <button v-for="emoji in EMOJI_LIST" :key="emoji" type="button"
                      @click="groupForm.icon = emoji"
                      class="w-7 h-7 flex items-center justify-center rounded text-base hover:bg-background transition-colors"
                      :class="groupForm.icon === emoji ? 'bg-background shadow-sm ring-1 ring-primary' : ''">
                {{ emoji }}
              </button>
            </div>
          </div>
          <div class="flex gap-2 pt-2">
            <button @click="createGroup" :disabled="!groupForm.name.trim()"
                    class="flex-1 py-2.5 rounded-lg bg-primary text-primary-foreground text-sm font-medium
                           hover:bg-primary/90 disabled:opacity-50 transition-colors">
              {{ editingGroup ? t('common.save') : t('common.create') }}
            </button>
            <button @click="showGroupModal = false; editingGroup = null; groupForm = { name: '', icon: '📝' }" class="px-4 py-2.5 text-sm text-muted-foreground hover:text-foreground">
              {{ t('common.cancel') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Delete note modal -->
    <div v-if="pendingDeleteId"
         class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
         @click.self="pendingDeleteId = null">
      <div class="w-full max-w-sm rounded-xl border border-border bg-card p-6 shadow-xl">
        <h2 class="text-lg font-semibold mb-1">{{ t('notes.deleteTitle') }}</h2>
        <p class="text-sm text-muted-foreground mb-1">
          {{ t('notes.deleteConfirm', { name: notes.find(n => n.id === pendingDeleteId)?.title }) }}
        </p>
        <p class="text-sm text-destructive mb-5">{{ t('common.irreversible') }}</p>
        <div class="flex gap-2">
          <button @click="confirmDeleteNote"
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
          {{ t('notes.deleteConfirm', { name: groups.find(g => g.id === pendingDeleteGroupId)?.name }) }}
        </p>
        <p class="text-sm text-destructive mb-5">{{ t('notes.groupItemsKept') }}</p>
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
