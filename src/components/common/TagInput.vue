<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import type { Tag } from '../../types'
import { X, Plus } from 'lucide-vue-next'

const props = defineProps<{
  modelValue: string[]
  placeholder?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [tags: string[]]
}>()

const { t } = useI18n()
const inputValue = ref('')
const suggestions = ref<Tag[]>([])
const showSuggestions = ref(false)
const inputRef = ref<HTMLInputElement>()

const filteredSuggestions = computed(() =>
  suggestions.value.filter(tg =>
    tg.name.toLowerCase().includes(inputValue.value.toLowerCase()) &&
    !props.modelValue.includes(tg.name)
  )
)

async function loadSuggestions() {
  if (!inputValue.value) {
    suggestions.value = []
    return
  }
  try {
    const all = await invoke<Tag[]>('list_tags')
    suggestions.value = all
  } catch {
    suggestions.value = []
  }
}

function addTag(name: string) {
  const trimmed = name.trim()
  if (!trimmed || props.modelValue.includes(trimmed)) return
  emit('update:modelValue', [...props.modelValue, trimmed])
  inputValue.value = ''
  showSuggestions.value = false
}

function removeTag(name: string) {
  emit('update:modelValue', props.modelValue.filter(tg => tg !== name))
}

function onInput() {
  loadSuggestions()
  showSuggestions.value = true
}

function onBlur() {
  setTimeout(() => {
    showSuggestions.value = false
  }, 200)
}
</script>

<template>
  <div class="relative">
    <div class="flex flex-wrap gap-1.5 p-2 rounded-lg border border-input bg-background min-h-[42px]">
      <span v-for="tag in modelValue" :key="tag"
            class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-primary/10 text-primary text-sm">
        {{ tag }}
        <button @click="removeTag(tag)" class="hover:text-destructive">
          <X class="w-3 h-3" />
        </button>
      </span>
      <input ref="inputRef"
             v-model="inputValue"
             @input="onInput"
             @focus="showSuggestions = true"
             @blur="onBlur"
             @keydown.enter.prevent="addTag(inputValue)"
             @keydown.backspace="!inputValue && modelValue.length && removeTag(modelValue[modelValue.length - 1])"
             :placeholder="modelValue.length ? '' : (placeholder || t('tagInput.placeholder'))"
             class="flex-1 min-w-[80px] bg-transparent text-sm outline-none placeholder:text-muted-foreground" />
    </div>

    <!-- Suggestions dropdown -->
    <div v-if="showSuggestions && filteredSuggestions.length > 0"
         class="absolute top-full left-0 right-0 mt-1 bg-popover border border-border rounded-lg shadow-lg overflow-hidden z-10">
      <button v-for="tag in filteredSuggestions" :key="tag.id"
              @mousedown.prevent="addTag(tag.name)"
              class="w-full px-3 py-2 text-sm text-left hover:bg-muted transition-colors flex items-center gap-2">
        <span class="w-2 h-2 rounded-full" :style="{ backgroundColor: tag.color || '#6366f1' }"></span>
        {{ tag.name }}
      </button>
    </div>

    <!-- Create new tag option -->
    <div v-if="showSuggestions && inputValue && !filteredSuggestions.find(tg => tg.name.toLowerCase() === inputValue.toLowerCase())"
         class="absolute top-full left-0 right-0 mt-1 bg-popover border border-border rounded-lg shadow-lg overflow-hidden z-10">
      <button @mousedown.prevent="addTag(inputValue)"
              class="w-full px-3 py-2 text-sm text-left hover:bg-muted transition-colors flex items-center gap-2 text-primary">
        <Plus class="w-4 h-4" />
        {{ t('tagInput.createTag') }} "{{ inputValue }}"
      </button>
    </div>
  </div>
</template>
