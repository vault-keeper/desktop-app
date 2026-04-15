<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useI18n } from 'vue-i18n'
import type { Bookmark, BookmarkGroup, Tag } from '../../types'
import { ArrowLeft, Save, Trash2, ExternalLink } from 'lucide-vue-next'
import TagInput from '../../components/TagInput.vue'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()

const bookmark = ref<Bookmark | null>(null)
const groups = ref<BookmarkGroup[]>([])
const loading = ref(true)
const saving = ref(false)
const showDeleteConfirm = ref(false)
const error = ref('')
const bookmarkTags = ref<Tag[]>([])

const editForm = ref({ title: '', url: '', description: '', group_id: '' })

const hasChanges = computed(() => {
  if (!bookmark.value) return false
  return (
    editForm.value.title       !== bookmark.value.title ||
    editForm.value.url         !== bookmark.value.url ||
    editForm.value.description !== (bookmark.value.description ?? '') ||
    editForm.value.group_id    !== (bookmark.value.group_id    ?? '')
  )
})

async function load() {
  loading.value = true
  try {
    ;[bookmark.value, groups.value] = await Promise.all([
      invoke<Bookmark>('get_bookmark', { id: route.params.id as string }),
      invoke<BookmarkGroup[]>('list_bookmark_groups'),
    ])
    editForm.value = {
      title:       bookmark.value.title,
      url:         bookmark.value.url,
      description: bookmark.value.description ?? '',
      group_id:    bookmark.value.group_id    ?? '',
    }
    bookmarkTags.value = await invoke<Tag[]>('get_entity_tags', {
      entityType: 'bookmark', entityId: route.params.id as string
    })
  } catch (e: any) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

async function save() {
  if (!bookmark.value || !hasChanges.value) return
  saving.value = true
  try {
    await invoke('update_bookmark', {
      id: bookmark.value.id,
      data: {
        title:       editForm.value.title,
        url:         editForm.value.url,
        description: editForm.value.description || undefined,
        group_id:    editForm.value.group_id    || undefined,
      }
    })
    bookmark.value = { ...bookmark.value, ...editForm.value }
  } catch (e: any) {
    error.value = String(e)
  } finally {
    saving.value = false
  }
}

function formatDate(iso: string) {
  const d = new Date(iso)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth()+1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

async function deleteBookmark() {
  if (!bookmark.value) return
  await invoke('delete_bookmark', { id: bookmark.value.id })
  router.back()
}

onMounted(load)
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
        <h1 class="text-xl font-semibold tracking-tight">{{ bookmark?.title || t('bookmarks.detailTitle') }}</h1>
      </div>
      <div class="flex items-center gap-1.5">
        <button v-if="bookmark" @click="openUrl(bookmark.url)"
                class="inline-flex items-center justify-center rounded-md h-9 w-9 text-muted-foreground
                       hover:bg-accent hover:text-foreground transition-colors"
                :title="t('common.openInBrowser')">
          <ExternalLink class="w-4 h-4" />
        </button>
        <button @click="showDeleteConfirm = true"
                class="inline-flex items-center justify-center rounded-md h-9 w-9 text-muted-foreground
                       hover:bg-destructive/10 hover:text-destructive transition-colors">
          <Trash2 class="w-4 h-4" />
        </button>
        <button @click="save" :disabled="!hasChanges || saving"
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
      <div v-for="i in 4" :key="i" class="h-10 rounded-md bg-muted animate-pulse" />
    </div>

    <!-- Error -->
    <div v-else-if="error"
         class="rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
      {{ error }}
    </div>

    <!-- Form -->
    <div v-else class="rounded-xl border border-border bg-card shadow-sm">
      <div class="p-6 space-y-5">

        <!-- Title -->
        <div class="space-y-2">
          <label class="text-sm font-medium leading-none">{{ t('bookmarks.titleLabel') }}</label>
          <input v-model="editForm.title" :placeholder="t('bookmarks.titlePlaceholder')"
                 class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                        placeholder:text-muted-foreground focus-visible:outline-none
                        focus-visible:ring-1 focus-visible:ring-ring transition-colors" />
        </div>

        <!-- URL -->
        <div class="space-y-2">
          <label class="text-sm font-medium leading-none">{{ t('common.url') }}</label>
          <input v-model="editForm.url" :placeholder="t('bookmarks.urlPlaceholder')"
                 class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                        placeholder:text-muted-foreground focus-visible:outline-none
                        focus-visible:ring-1 focus-visible:ring-ring transition-colors" />
        </div>

        <!-- Group -->
        <div class="space-y-2">
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

        <!-- Description -->
        <div class="space-y-2">
          <label class="text-sm font-medium leading-none">{{ t('bookmarks.descLabel') }}</label>
          <textarea v-model="editForm.description" :placeholder="t('bookmarks.descPlaceholder')" rows="3"
                    class="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                           placeholder:text-muted-foreground focus-visible:outline-none
                           focus-visible:ring-1 focus-visible:ring-ring resize-none transition-colors" />
        </div>

        <!-- Tags -->
        <div class="space-y-2">
          <label class="text-sm font-medium leading-none">{{ t('common.tags') }}</label>
          <TagInput v-model="bookmarkTags" entity-type="bookmark" :entity-id="bookmark?.id ?? ''" />
        </div>

      </div>

      <!-- Footer meta -->
      <div v-if="bookmark" class="border-t border-border px-6 py-3">
        <p class="text-xs text-muted-foreground">
          {{ t('common.createdAt') }}{{ formatDate(bookmark.created_at) }}
        </p>
      </div>
    </div>

    <!-- Delete dialog -->
    <div v-if="showDeleteConfirm"
         class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
         @click.self="showDeleteConfirm = false">
      <div class="w-full max-w-sm rounded-xl border border-border bg-card p-6 shadow-xl">
        <h2 class="text-lg font-semibold mb-1">{{ t('bookmarks.deleteTitle') }}</h2>
        <p class="text-sm text-muted-foreground mb-1">
          {{ t('bookmarks.deleteConfirm', { name: bookmark?.title }) }}
        </p>
        <p class="text-sm text-destructive mb-5">{{ t('common.irreversible') }}</p>
        <div class="flex gap-2">
          <button @click="deleteBookmark"
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
