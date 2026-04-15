<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import type { Tag } from '../../types'
import { Plus, Tag as TagIcon, Trash2, Edit3, Lock } from 'lucide-vue-next'

const { t } = useI18n()

const tags = ref<Tag[]>([])
const loading = ref(true)
const showCreateModal = ref(false)
const editingId = ref<string | null>(null)
const editError = ref('')
const deleteError = ref('')
const form = ref({ name: '', color: '', icon: '' })
const editForm = ref({ name: '', color: '', icon: '' })

// 24-colour palette
const COLOR_PALETTE = [
  '#ef4444', '#f97316', '#f59e0b', '#eab308',
  '#84cc16', '#22c55e', '#10b981', '#14b8a6',
  '#06b6d4', '#0ea5e9', '#3b82f6', '#6366f1',
  '#8b5cf6', '#a855f7', '#d946ef', '#ec4899',
  '#f43f5e', '#fb923c', '#fbbf24', '#a3e635',
  '#34d399', '#22d3ee', '#60a5fa', '#818cf8',
]

function randomColor() {
  return COLOR_PALETTE[Math.floor(Math.random() * COLOR_PALETTE.length)]
}

async function loadTags() {
  loading.value = true
  try {
    tags.value = await invoke<Tag[]>('list_tags')
  } catch (e) {
    console.error('Failed to load tags:', e)
  } finally {
    loading.value = false
  }
}

function openCreate() {
  form.value = { name: '', color: randomColor(), icon: '' }
  showCreateModal.value = true
}

async function createTag() {
  if (!form.value.name.trim()) return
  try {
    await invoke('create_tag', { data: { name: form.value.name.trim(), color: form.value.color, icon: null } })
    showCreateModal.value = false
    await loadTags()
  } catch (e) {
    console.error('Failed to create tag:', e)
  }
}

async function deleteTag(id: string) {
  deleteError.value = ''
  try {
    await invoke('delete_tag', { id })
    await loadTags()
  } catch (e: any) {
    deleteError.value = String(e)
    setTimeout(() => { deleteError.value = '' }, 3000)
  }
}

async function updateTag(id: string) {
  if (!editForm.value.name.trim()) return
  editError.value = ''
  try {
    await invoke('update_tag', { id, data: { name: editForm.value.name.trim(), color: editForm.value.color, icon: null } })
    editingId.value = null
    await loadTags()
  } catch (e: any) {
    editError.value = String(e)
  }
}

function startEdit(tag: Tag) {
  editingId.value = tag.id
  editError.value = ''
  editForm.value = { name: tag.name, color: tag.color || randomColor(), icon: tag.icon || '' }
}

loadTags()
</script>

<template>
  <div class="max-w-5xl mx-auto">
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold">{{ t('tags.title') }}</h1>
      <button @click="openCreate"
              class="flex items-center gap-1.5 px-3 py-2 rounded-lg bg-primary
                     text-primary-foreground text-sm hover:bg-primary/90 transition-colors">
        <Plus class="w-4 h-4" />
        {{ t('tags.newTag') }}
      </button>
    </div>

    <!-- Delete error toast -->
    <div v-if="deleteError"
         class="mb-4 rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-2.5
                text-sm text-destructive">
      {{ deleteError }}
    </div>

    <div v-if="loading" class="space-y-2">
      <div v-for="i in 5" :key="i" class="h-14 bg-muted/50 rounded-lg animate-pulse" />
    </div>

    <div v-else-if="tags.length === 0" class="text-center py-16 text-muted-foreground">
      <TagIcon class="w-12 h-12 mx-auto mb-4 opacity-40" />
      <p class="text-lg font-medium">{{ t('tags.empty') }}</p>
      <p class="text-sm mt-1">{{ t('tags.emptyPrompt') }}</p>
    </div>

    <div v-else class="space-y-2">
      <div v-for="tag in tags" :key="tag.id"
           class="bg-card rounded-xl border border-border overflow-hidden group">

        <!-- View mode -->
        <div v-if="editingId !== tag.id" class="flex items-center gap-3 p-4">
          <!-- Color dot -->
          <div class="w-3.5 h-3.5 rounded-full shrink-0"
               :style="{ backgroundColor: tag.color ?? '#6366f1' }" />
          <!-- Name pill -->
          <span class="px-2.5 py-0.5 rounded-full text-xs font-semibold text-white"
                :style="{ backgroundColor: tag.color ?? '#6366f1' }">
            #{{ tag.name }}
          </span>
          <!-- Usage badge -->
          <span v-if="tag.usage_count > 0"
                class="text-xs text-muted-foreground bg-muted px-2 py-0.5 rounded-full">
            {{ t('tags.usedCount') }} {{ tag.usage_count }} {{ t('tags.timesSuffix') }}
          </span>
          <span v-else class="text-xs text-muted-foreground">{{ t('tags.unused') }}</span>

          <!-- Actions -->
          <div class="ml-auto flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <template v-if="tag.usage_count > 0">
              <span class="flex items-center gap-1 px-2 py-1 rounded-md text-xs
                           text-muted-foreground bg-muted/50 cursor-not-allowed"
                    :title="t('tags.lockedTitle')">
                <Lock class="w-3 h-3" />
                {{ t('nav.lock') }}
              </span>
            </template>
            <template v-else>
              <button @click="startEdit(tag)"
                      class="p-1.5 rounded-md hover:bg-muted transition-colors">
                <Edit3 class="w-4 h-4 text-muted-foreground" />
              </button>
              <button @click="deleteTag(tag.id)"
                      class="p-1.5 rounded-md hover:bg-destructive/10
                             hover:text-destructive transition-colors">
                <Trash2 class="w-4 h-4" />
              </button>
            </template>
          </div>
        </div>

        <!-- Edit mode -->
        <div v-else class="p-4 space-y-3 bg-muted/20">
          <div class="flex gap-2">
            <input v-model="editForm.name" autofocus
                   class="flex-1 h-9 px-3 rounded-md border border-input bg-background
                          text-sm focus-visible:outline-none focus-visible:ring-1
                          focus-visible:ring-ring" />
            <button @click="updateTag(tag.id)" :disabled="!editForm.name.trim()"
                    class="h-9 px-3 rounded-md bg-primary text-primary-foreground text-xs
                           disabled:opacity-50 hover:bg-primary/90 transition-colors">
              {{ t('common.save') }}
            </button>
            <button @click="editingId = null; editError = ''"
                    class="h-9 px-3 rounded-md border border-input text-xs
                           hover:bg-muted transition-colors">
              {{ t('common.cancel') }}
            </button>
          </div>
          <!-- Colour picker -->
          <div class="grid grid-cols-12 gap-1.5">
            <button
              v-for="c in COLOR_PALETTE" :key="c"
              type="button"
              class="w-6 h-6 rounded-full border-2 transition-all hover:scale-110"
              :class="editForm.color === c ? 'border-foreground scale-110' : 'border-transparent'"
              :style="{ backgroundColor: c }"
              @click="editForm.color = c"
            />
          </div>
          <p v-if="editError" class="text-xs text-destructive">{{ editError }}</p>
        </div>
      </div>
    </div>

    <!-- Create Modal -->
    <div v-if="showCreateModal"
         class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
         @click.self="showCreateModal = false">
      <div class="bg-card rounded-xl border border-border p-6 w-full max-w-sm shadow-xl">
        <h2 class="text-lg font-semibold mb-4">{{ t('tags.createTitle') }}</h2>
        <div class="space-y-4">
          <div class="space-y-2">
            <label class="text-sm font-medium">{{ t('tags.nameLabel') }}</label>
            <input v-model="form.name" :placeholder="t('tags.namePlaceholder')" autofocus
                   class="flex h-10 w-full rounded-md border border-input bg-background
                          px-3 py-2 text-sm placeholder:text-muted-foreground
                          focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
          </div>
          <!-- Preview pill -->
          <div v-if="form.name" class="flex items-center gap-2">
            <span class="text-xs text-muted-foreground">{{ t('tags.preview') }}</span>
            <span class="px-2.5 py-0.5 rounded-full text-xs font-semibold text-white"
                  :style="{ backgroundColor: form.color }">
              #{{ form.name }}
            </span>
          </div>
          <!-- Color picker -->
          <div class="space-y-2">
            <label class="text-sm font-medium">{{ t('common.color') }}</label>
            <div class="grid grid-cols-12 gap-1.5">
              <button
                v-for="c in COLOR_PALETTE" :key="c"
                type="button"
                class="w-6 h-6 rounded-full border-2 transition-all hover:scale-110"
                :class="form.color === c ? 'border-foreground scale-110' : 'border-transparent'"
                :style="{ backgroundColor: c }"
                @click="form.color = c"
              />
            </div>
          </div>
          <div class="flex gap-2 pt-2">
            <button @click="createTag" :disabled="!form.name.trim()"
                    class="flex-1 h-10 rounded-md bg-primary text-primary-foreground text-sm
                           font-medium hover:bg-primary/90 disabled:opacity-50 transition-colors">
              {{ t('common.create') }}
            </button>
            <button @click="showCreateModal = false"
                    class="px-4 h-10 rounded-md border border-input text-sm
                           hover:bg-muted transition-colors">
              {{ t('common.cancel') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
