<script setup lang="ts">
/**
 * TagInput — reusable inline tag selector / creator.
 *
 * Usage (existing entity, auto-syncs to backend):
 *   <TagInput v-model="tags" entity-type="account" :entity-id="account.id" />
 *
 * Usage (new entity, parent applies tags after creation):
 *   <TagInput v-model="pendingTags" />
 *
 * Keyboard shortcuts:
 *   Enter / Space  — add typed tag (exact match selects existing, else creates new)
 *   Backspace      — remove last tag when input is empty
 *   Escape         — close dropdown / clear input
 */

import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import type { Tag } from '../types'
import { X, Plus } from 'lucide-vue-next'

const props = defineProps<{
  modelValue: Tag[]
  entityType?: string
  entityId?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [tags: Tag[]]
}>()

// ── color palette (24 colours) ─────────────────────────────────
const TAG_COLORS = [
  '#ef4444', '#f97316', '#f59e0b', '#eab308',
  '#84cc16', '#22c55e', '#10b981', '#14b8a6',
  '#06b6d4', '#0ea5e9', '#3b82f6', '#6366f1',
  '#8b5cf6', '#a855f7', '#d946ef', '#ec4899',
  '#f43f5e', '#fb923c', '#fbbf24', '#a3e635',
  '#34d399', '#22d3ee', '#60a5fa', '#818cf8',
]
function randomColor() {
  return TAG_COLORS[Math.floor(Math.random() * TAG_COLORS.length)]
}

const { t } = useI18n()
const allTags = ref<Tag[]>([])
const inputVal = ref('')
const showDropdown = ref(false)
const inputRef = ref<HTMLInputElement>()

// Dropdown: exclude already-selected, filter by typed text
const filtered = computed(() => {
  const selectedIds = new Set(props.modelValue.map(t => t.id))
  const q = inputVal.value.replace(/^#/, '').toLowerCase().trim()
  return allTags.value
    .filter(t => !selectedIds.has(t.id) && (q === '' || t.name.toLowerCase().includes(q)))
    .slice(0, 8)
})

// Whether user's exact input already exists as a tag name
const exactExists = computed(() => {
  const q = inputVal.value.replace(/^#/, '').trim().toLowerCase()
  return q === '' || allTags.value.some(t => t.name.toLowerCase() === q)
})

const queryText = computed(() => inputVal.value.replace(/^#/, '').trim())

async function loadTags() {
  try { allTags.value = await invoke<Tag[]>('list_tags') } catch {}
}

async function addTag(tag: Tag) {
  if (props.modelValue.find(t => t.id === tag.id)) return
  emit('update:modelValue', [...props.modelValue, tag])
  if (props.entityType && props.entityId) {
    await invoke('tag_entity', {
      tagId: tag.id,
      entityType: props.entityType,
      entityId: props.entityId,
    }).catch(() => {})
  }
  inputVal.value = ''
}

async function removeTag(tag: Tag) {
  emit('update:modelValue', props.modelValue.filter(t => t.id !== tag.id))
  if (props.entityType && props.entityId) {
    await invoke('untag_entity', {
      tagId: tag.id,
      entityType: props.entityType,
      entityId: props.entityId,
    }).catch(() => {})
  }
}

async function commitInput() {
  const name = queryText.value
  if (!name) return

  // Match existing tag (case-insensitive)
  const existing = allTags.value.find(t => t.name.toLowerCase() === name.toLowerCase())
  if (existing) {
    await addTag(existing)
    return
  }

  // Create new tag with random colour
  try {
    const tag = await invoke<Tag>('create_tag', {
      data: { name, color: randomColor(), icon: null }
    })
    allTags.value.push(tag)
    await addTag(tag)
  } catch (e) {
    console.error('Failed to create tag:', e)
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    e.preventDefault()
    commitInput()
  } else if (e.key === ' ' && queryText.value) {
    e.preventDefault()
    commitInput()
  } else if (e.key === 'Backspace' && !inputVal.value && props.modelValue.length) {
    removeTag(props.modelValue[props.modelValue.length - 1])
  } else if (e.key === 'Escape') {
    inputVal.value = ''
    showDropdown.value = false
    inputRef.value?.blur()
  }
}

function onBlur() {
  // Delay so mousedown on dropdown items fires first
  setTimeout(() => { showDropdown.value = false }, 160)
}

onMounted(loadTags)
</script>

<template>
  <div class="relative">
    <!-- Tag pills + input row -->
    <div
      class="flex flex-wrap items-center gap-1.5 min-h-10 px-3 py-2 rounded-md
             border border-input bg-background cursor-text"
      @click="inputRef?.focus(); showDropdown = true"
    >
      <!-- Selected tag pills -->
      <span
        v-for="tag in modelValue"
        :key="tag.id"
        class="inline-flex items-center gap-1 pl-2 pr-1 py-0.5 rounded-full
               text-xs font-medium text-white select-none"
        :style="{ backgroundColor: tag.color ?? '#6366f1' }"
      >
        <span class="opacity-75">#</span>{{ tag.name }}
        <button
          type="button"
          class="ml-0.5 rounded-full hover:bg-white/20 p-0.5 transition-colors"
          @click.stop="removeTag(tag)"
        >
          <X class="w-2.5 h-2.5" />
        </button>
      </span>

      <!-- Text input -->
      <input
        ref="inputRef"
        v-model="inputVal"
        type="text"
        :placeholder="t('tagInput.placeholder')"
        class="flex-1 min-w-[8rem] bg-transparent text-sm outline-none
               placeholder:text-muted-foreground"
        @keydown="onKeydown"
        @focus="showDropdown = true"
        @blur="onBlur"
      />
    </div>

    <!-- Dropdown -->
    <div
      v-if="showDropdown && (filtered.length > 0 || queryText)"
      class="absolute left-0 right-0 z-20 mt-1 rounded-lg border border-border
             bg-card shadow-lg overflow-hidden"
    >
      <!-- Existing matching tags -->
      <button
        v-for="tag in filtered"
        :key="tag.id"
        type="button"
        class="flex items-center gap-2.5 w-full px-3 py-2 text-sm text-left
               hover:bg-muted transition-colors"
        @mousedown.prevent="addTag(tag)"
      >
        <span
          class="w-2.5 h-2.5 rounded-full shrink-0"
          :style="{ backgroundColor: tag.color ?? '#6366f1' }"
        />
        <span class="text-muted-foreground">#</span>{{ tag.name }}
      </button>

      <!-- "Create" option when no exact match -->
      <button
        v-if="queryText && !exactExists"
        type="button"
        class="flex items-center gap-2.5 w-full px-3 py-2 text-sm text-left
               hover:bg-muted transition-colors text-muted-foreground
               border-t border-border"
        @mousedown.prevent="commitInput"
      >
        <Plus class="w-3.5 h-3.5 shrink-0" />
        {{ t('tagInput.createTag') }} &ldquo;<span class="font-medium text-foreground">{{ queryText }}</span>&rdquo;
      </button>
    </div>
  </div>
</template>
