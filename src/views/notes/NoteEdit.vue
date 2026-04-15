<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import type { Note, NoteGroup, Tag } from '../../types'
import { ArrowLeft, Save, Trash2, Lock, Unlock } from 'lucide-vue-next'
import MarkdownEditor from '../../components/MarkdownEditor.vue'
import TagInput from '../../components/TagInput.vue'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()

const note = ref<Note | null>(null)
const groups = ref<NoteGroup[]>([])
const loading = ref(true)
const saving = ref(false)
const showDeleteConfirm = ref(false)
const showEncryptionInput = ref(false)
const secondaryPassword = ref('')
const isDecrypted = ref(false)
const encryptError = ref('')
const noteTags = ref<Tag[]>([])
/// Holds the secondary password used to decrypt this note in the current
/// edit session, so we can re-encrypt transparently on save.
const currentDecryptPassword = ref('')

const editForm = ref({ title: '', content: '', group_id: '' })

const hasChanges = computed(() =>
  note.value && (
    editForm.value.title !== note.value.title ||
    editForm.value.content !== note.value.content ||
    editForm.value.group_id !== (note.value.group_id ?? '')
  )
)

async function loadNote() {
  loading.value = true
  currentDecryptPassword.value = ''
  isDecrypted.value = false
  try {
    ;[note.value, groups.value] = await Promise.all([
      invoke<Note>('get_note', { id: route.params.id }),
      invoke<NoteGroup[]>('list_note_groups'),
    ])
    editForm.value = {
      title: note.value.title,
      content: note.value.content,
      group_id: note.value.group_id ?? '',
    }
    isDecrypted.value = !note.value.is_encrypted
    noteTags.value = await invoke<Tag[]>('get_entity_tags', {
      entityType: 'note', entityId: route.params.id as string,
    })
  } catch (e) {
    console.error('Failed to load note:', e)
  } finally {
    loading.value = false
  }
}

async function saveNote() {
  if (!note.value || !hasChanges.value) return
  saving.value = true
  try {
    if (note.value.is_encrypted && isDecrypted.value && currentDecryptPassword.value) {
      // Note was encrypted and decrypted in the editor — re-encrypt with same password.
      await invoke('save_and_reencrypt_note', {
        id: note.value.id,
        title: editForm.value.title,
        content: editForm.value.content,
        groupId: editForm.value.group_id || null,
        secondaryPassword: currentDecryptPassword.value,
      })
      note.value = await invoke<Note>('get_note', { id: note.value.id as string })
      // Reset to locked state — note is re-encrypted in the DB.
      isDecrypted.value = false
      editForm.value.content = ''
      currentDecryptPassword.value = ''
    } else {
      // Normal save.  For encrypted notes only title/group can change via this path.
      const updateData: Record<string, unknown> = {
        title: editForm.value.title,
        group_id: editForm.value.group_id || null,
      }
      if (!note.value.is_encrypted) {
        updateData.content = editForm.value.content
      }
      await invoke('update_note', { id: note.value.id, data: updateData })
      note.value = {
        ...note.value,
        title: editForm.value.title,
        group_id: editForm.value.group_id || null,
        ...(note.value.is_encrypted ? {} : { content: editForm.value.content }),
      }
    }
  } catch (e) {
    console.error('Failed to save note:', e)
  } finally {
    saving.value = false
  }
}

async function deleteNote() {
  if (!note.value) return
  await invoke('delete_note', { id: note.value.id })
  router.replace({ name: 'notes' })
}

async function encryptNote() {
  if (!note.value || !secondaryPassword.value) return
  encryptError.value = ''
  try {
    await invoke('encrypt_note', { id: note.value.id, secondaryPassword: secondaryPassword.value })
    note.value = { ...note.value, is_encrypted: 1, content: '' }
    editForm.value.content = ''
    isDecrypted.value = false
    showEncryptionInput.value = false
    secondaryPassword.value = ''
    currentDecryptPassword.value = ''
  } catch (e: any) {
    encryptError.value = String(e)
  }
}

async function decryptNote() {
  if (!note.value || !secondaryPassword.value) return
  encryptError.value = ''
  try {
    const decrypted = await invoke<Note>('decrypt_note', { id: note.value.id, secondaryPassword: secondaryPassword.value })
    editForm.value.content = decrypted.content
    isDecrypted.value = true
    showEncryptionInput.value = false
    currentDecryptPassword.value = secondaryPassword.value  // keep for re-encryption on save
    secondaryPassword.value = ''
  } catch {
    encryptError.value = t('notes.wrongPassword')
  }
}

function formatDate(iso: string) {
  const d = new Date(iso)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth()+1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

onMounted(loadNote)
</script>

<template>
  <div class="max-w-3xl mx-auto">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div class="flex items-center gap-3">
        <button @click="router.back()"
                class="inline-flex items-center justify-center rounded-md h-9 w-9
                       text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors">
          <ArrowLeft class="w-4 h-4" />
        </button>
        <h1 class="text-xl font-semibold tracking-tight">
          {{ note?.title || t('notes.editTitle') }}
        </h1>
        <span v-if="note?.is_encrypted"
              class="px-2 py-0.5 rounded-md text-xs font-medium bg-amber-100 text-amber-700
                     dark:bg-amber-900/30 dark:text-amber-400">
          🔒 {{ t('notes.encrypted') }}
        </span>
      </div>
      <div class="flex items-center gap-1.5">
        <button v-if="note?.is_encrypted && !isDecrypted"
                @click="showEncryptionInput = true"
                class="inline-flex items-center gap-1.5 rounded-md border border-input h-9 px-3 text-sm
                       text-muted-foreground hover:bg-accent hover:text-foreground transition-colors">
          <Unlock class="w-4 h-4" /> {{ t('notes.decryptAction') }}
        </button>
        <button v-if="!note?.is_encrypted"
                @click="showEncryptionInput = true"
                class="inline-flex items-center gap-1.5 rounded-md border border-input h-9 px-3 text-sm
                       text-muted-foreground hover:bg-accent hover:text-foreground transition-colors">
          <Lock class="w-4 h-4" /> {{ t('notes.encryptAction') }}
        </button>
        <button @click="showDeleteConfirm = true"
                class="inline-flex items-center justify-center rounded-md h-9 w-9
                       text-muted-foreground hover:bg-destructive/10 hover:text-destructive transition-colors">
          <Trash2 class="w-4 h-4" />
        </button>
        <button @click="saveNote" :disabled="!hasChanges || saving"
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

    <!-- Form card -->
    <div v-else-if="note" class="rounded-xl border border-border bg-card shadow-sm">
      <div class="p-6 space-y-5">
        <div class="space-y-2">
          <label class="text-sm font-medium leading-none">{{ t('common.title') }}</label>
          <input v-model="editForm.title" :placeholder="t('notes.titlePlaceholder')"
                 class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                        placeholder:text-muted-foreground focus-visible:outline-none
                        focus-visible:ring-1 focus-visible:ring-ring transition-colors" />
        </div>
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
        <div class="space-y-2">
          <label class="text-sm font-medium leading-none">{{ t('common.tags') }}</label>
          <TagInput v-model="noteTags" entity-type="note" :entity-id="note.id" />
        </div>
        <div class="space-y-2">
          <label class="text-sm font-medium leading-none">{{ t('common.content') }}</label>
          <div v-if="note.is_encrypted && !isDecrypted"
               class="flex flex-col items-center justify-center gap-4 rounded-md border border-input
                      bg-muted/30 py-16 text-muted-foreground">
            <Lock class="w-10 h-10 opacity-30" />
            <p class="text-sm font-medium">{{ t('notes.encryptedMsg') }}</p>
            <button @click="showEncryptionInput = true"
                    class="inline-flex items-center gap-1.5 rounded-md bg-primary px-4 h-9 text-sm
                           font-medium text-primary-foreground hover:bg-primary/90 transition-colors">
              <Unlock class="w-4 h-4" /> {{ t('notes.decryptBtn') }}
            </button>
          </div>
          <div v-else class="h-[480px] rounded-md border border-input overflow-hidden">
            <MarkdownEditor v-model="editForm.content"
                            :placeholder="t('notes.contentPlaceholder')"
                            variant="flat"
                            class="h-full" />
          </div>
        </div>
      </div>
      <div class="border-t border-border px-6 py-3">
        <p class="text-xs text-muted-foreground">
          {{ t('common.createdAt') }}{{ formatDate(note.created_at) }}
          · {{ t('common.updatedAt') }}{{ formatDate(note.updated_at) }}
        </p>
      </div>
    </div>

    <!-- Encryption modal -->
    <div v-if="showEncryptionInput"
         class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
         @click.self="showEncryptionInput = false; secondaryPassword = ''; encryptError = ''">
      <div class="w-full max-w-sm rounded-xl border border-border bg-card p-6 shadow-xl">
        <h2 class="text-lg font-semibold mb-4">{{ note?.is_encrypted ? t('notes.decryptTitle') : t('notes.encryptTitle') }}</h2>
        <div class="space-y-3">
          <input v-model="secondaryPassword" type="password"
                 :placeholder="note?.is_encrypted ? t('notes.decryptPlaceholder') : t('notes.encryptPlaceholder')"
                 autofocus
                 class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                        placeholder:text-muted-foreground focus-visible:outline-none
                        focus-visible:ring-1 focus-visible:ring-ring transition-colors"
                 @keydown.enter="note?.is_encrypted ? decryptNote() : encryptNote()" />
          <p v-if="encryptError" class="text-sm text-destructive">{{ encryptError }}</p>
          <div class="flex gap-2">
            <button @click="note?.is_encrypted ? decryptNote() : encryptNote()"
                    :disabled="!secondaryPassword"
                    class="flex-1 h-9 rounded-md bg-primary text-primary-foreground text-sm font-medium
                           hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50 transition-colors">
              {{ note?.is_encrypted ? t('notes.decryptAction') : t('notes.encryptAction') }}
            </button>
            <button @click="showEncryptionInput = false; secondaryPassword = ''; encryptError = ''"
                    class="flex-1 h-9 rounded-md border border-input bg-background text-sm
                           hover:bg-accent transition-colors">
              {{ t('common.cancel') }}
            </button>
          </div>
        </div>
      </div>
    </div>

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
