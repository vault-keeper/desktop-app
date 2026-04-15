<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import MarkdownIt from 'markdown-it'
import type { Note, NoteGroup, Tag } from '../../types'
import { ArrowLeft, Edit2, Trash2, Lock, Unlock } from 'lucide-vue-next'

const md = new MarkdownIt({ breaks: true, linkify: true })
const route = useRoute()
const router = useRouter()
const { t } = useI18n()

const note = ref<Note | null>(null)
const noteTags = ref<Tag[]>([])
const loading = ref(true)
const showDeleteConfirm = ref(false)
const isDecrypted = ref(false)
const decryptedContent = ref('')
const showPasswordInput = ref(false)
const password = ref('')
const passwordError = ref('')
const decrypting = ref(false)

const renderedHtml = computed(() =>
  md.render(isDecrypted.value ? decryptedContent.value : (note.value?.content ?? ''))
)

async function load() {
  loading.value = true
  try {
    note.value = await invoke<Note>('get_note', { id: route.params.id })
    noteTags.value = await invoke<Tag[]>('get_entity_tags', {
      entityType: 'note', entityId: route.params.id as string,
    })
    isDecrypted.value = !note.value.is_encrypted
    decryptedContent.value = note.value.content
  } catch (e) {
    console.error('Failed to load note:', e)
  } finally {
    loading.value = false
  }
}

async function decrypt() {
  if (!note.value || !password.value) return
  decrypting.value = true
  passwordError.value = ''
  try {
    const result = await invoke<Note>('decrypt_note', {
      id: note.value.id,
      secondaryPassword: password.value,
    })
    decryptedContent.value = result.content
    isDecrypted.value = true
    showPasswordInput.value = false
    password.value = ''
  } catch {
    passwordError.value = t('notes.wrongPassword')
  } finally {
    decrypting.value = false
  }
}

async function deleteNote() {
  if (!note.value) return
  await invoke('delete_note', { id: note.value.id })
  router.back()
}

function formatDate(iso: string) {
  const d = new Date(iso)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth()+1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

onMounted(load)
</script>

<template>
  <div class="max-w-3xl mx-auto">

    <!-- Loading -->
    <div v-if="loading" class="space-y-4">
      <div class="h-8 w-1/2 bg-muted/50 rounded animate-pulse" />
      <div v-for="i in 6" :key="i" class="h-5 bg-muted/50 rounded animate-pulse" />
    </div>

    <template v-else-if="note">
      <!-- Header bar -->
      <div class="flex items-center justify-between mb-8">
        <button @click="router.back()"
                class="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors">
          <ArrowLeft class="w-4 h-4" />
          {{ t('common.back') }}
        </button>
        <div class="flex items-center gap-1.5">
          <button @click="router.push({ name: 'note-edit', params: { id: note.id } })"
                  class="inline-flex items-center gap-1.5 px-3 h-8 rounded-md border border-input text-sm
                         text-muted-foreground hover:bg-accent hover:text-foreground transition-colors">
            <Edit2 class="w-3.5 h-3.5" />
            {{ t('common.edit') }}
          </button>
          <button @click="showDeleteConfirm = true"
                  class="inline-flex items-center justify-center h-8 w-8 rounded-md
                         text-muted-foreground hover:bg-destructive/10 hover:text-destructive transition-colors">
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Title -->
      <h1 class="text-3xl font-bold tracking-tight mb-4">{{ note.title }}</h1>

      <!-- Meta: tags + time -->
      <div class="flex flex-wrap items-center gap-3 mb-8 text-sm text-muted-foreground">
        <div v-if="noteTags.length" class="flex flex-wrap gap-1.5">
          <span v-for="tag in noteTags" :key="tag.id"
                class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-semibold text-white"
                :style="{ backgroundColor: tag.color ?? '#6366f1' }">
            #{{ tag.name }}
          </span>
        </div>
        <span v-if="noteTags.length" class="text-border">·</span>
        <span>{{ t('common.createdAt') }}{{ formatDate(note.created_at) }}</span>
        <template v-if="note.updated_at !== note.created_at">
          <span class="text-border">·</span>
          <span>{{ t('common.updatedAt') }}{{ formatDate(note.updated_at) }}</span>
        </template>
        <span v-if="note.is_encrypted"
              class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-xs font-medium
                     bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400">
          <Lock class="w-3 h-3" />
          {{ t('notes.encrypted') }}
        </span>
      </div>

      <hr class="border-border mb-8" />

      <!-- Encrypted & locked -->
      <div v-if="note.is_encrypted && !isDecrypted"
           class="flex flex-col items-center gap-5 py-20 text-muted-foreground">
        <Lock class="w-14 h-14 opacity-20" />
        <p class="text-sm font-medium">{{ t('notes.encryptedMsg') }}</p>
        <div v-if="!showPasswordInput">
          <button @click="showPasswordInput = true"
                  class="inline-flex items-center gap-1.5 px-4 h-9 rounded-lg bg-primary
                         text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors">
            <Unlock class="w-4 h-4" />
            {{ t('notes.detailDecryptBtn') }}
          </button>
        </div>
        <div v-else class="w-full max-w-xs space-y-2">
          <input v-model="password" type="password" :placeholder="t('notes.detailDecryptPlaceholder')" autofocus
                 class="w-full px-4 py-2 rounded-lg border border-input bg-background text-sm
                        focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                 @keydown.enter="decrypt" />
          <p v-if="passwordError" class="text-xs text-destructive">{{ passwordError }}</p>
          <button @click="decrypt" :disabled="!password || decrypting"
                  class="w-full h-9 rounded-lg bg-primary text-primary-foreground text-sm font-medium
                         hover:bg-primary/90 disabled:opacity-50 transition-colors">
            {{ decrypting ? t('notes.decryptingBtn') : t('common.confirm') }}
          </button>
        </div>
      </div>

      <!-- Article content -->
      <div v-else
           class="prose prose-sm max-w-none"
           v-html="renderedHtml" />
    </template>

    <!-- Delete modal -->
    <div v-if="showDeleteConfirm"
         class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
         @click.self="showDeleteConfirm = false">
      <div class="w-full max-w-sm rounded-xl border border-border bg-card p-6 shadow-xl">
        <h2 class="text-lg font-semibold mb-1">{{ t('notes.deleteTitle') }}</h2>
        <p class="text-sm text-muted-foreground mb-1">
          {{ t('notes.deleteConfirm', { name: note?.title }) }}
        </p>
        <p class="text-sm text-destructive mb-5">{{ t('common.irreversible') }}</p>
        <div class="flex gap-2">
          <button @click="deleteNote"
                  class="flex-1 h-9 rounded-md bg-destructive text-destructive-foreground text-sm font-medium hover:bg-destructive/90 transition-colors">
            {{ t('common.confirmDelete') }}
          </button>
          <button @click="showDeleteConfirm = false"
                  class="flex-1 h-9 rounded-md border border-input bg-background text-sm hover:bg-accent transition-colors">
            {{ t('common.cancel') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
