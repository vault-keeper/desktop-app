<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import MarkdownIt from 'markdown-it'
import { Bold, Italic, Heading2, Code, Link, List, ListOrdered, Quote, Eye, Edit2, Columns } from 'lucide-vue-next'

const props = defineProps<{ modelValue: string; placeholder?: string; variant?: 'default' | 'flat' }>()
const emit = defineEmits<{ (e: 'update:modelValue', v: string): void }>()

const { t } = useI18n()
const md = new MarkdownIt({ breaks: true, linkify: true })

type ViewMode = 'edit' | 'preview' | 'split'
const mode = ref<ViewMode>('split')
const textareaRef = ref<HTMLTextAreaElement | null>(null)

const rendered = computed(() => md.render(props.modelValue || ''))

function onInput(e: Event) {
  emit('update:modelValue', (e.target as HTMLTextAreaElement).value)
}

// Insert formatting around selection or at cursor
function wrap(before: string, after: string, placeholder: string) {
  const ta = textareaRef.value
  if (!ta) return
  const start = ta.selectionStart
  const end = ta.selectionEnd
  const selected = ta.value.slice(start, end) || placeholder
  const newVal = ta.value.slice(0, start) + before + selected + after + ta.value.slice(end)
  emit('update:modelValue', newVal)
  // Restore selection after Vue updates the value
  setTimeout(() => {
    ta.focus()
    ta.setSelectionRange(start + before.length, start + before.length + selected.length)
  }, 0)
}

function insertLine(prefix: string, placeholder: string) {
  const ta = textareaRef.value
  if (!ta) return
  const start = ta.selectionStart
  const lineStart = ta.value.lastIndexOf('\n', start - 1) + 1
  const newVal = ta.value.slice(0, lineStart) + prefix + ta.value.slice(lineStart)
  emit('update:modelValue', newVal)
  setTimeout(() => {
    ta.focus()
    const pos = lineStart + prefix.length + placeholder.length
    ta.setSelectionRange(lineStart + prefix.length, pos)
  }, 0)
}

const tools = computed(() => [
  { icon: Bold,        title: t('markdown.bold'),         action: () => wrap('**', '**', t('markdown.boldPlaceholder')) },
  { icon: Italic,      title: t('markdown.italic'),       action: () => wrap('*', '*', t('markdown.italicPlaceholder')) },
  { icon: Heading2,    title: t('markdown.heading'),      action: () => insertLine('## ', t('markdown.headingPlaceholder')) },
  { icon: Code,        title: t('markdown.code'),         action: () => wrap('`', '`', t('markdown.codePlaceholder')) },
  { icon: Quote,       title: t('markdown.quote'),        action: () => insertLine('> ', t('markdown.quotePlaceholder')) },
  { icon: List,        title: t('markdown.bulletList'),   action: () => insertLine('- ', t('markdown.listItem')) },
  { icon: ListOrdered, title: t('markdown.orderedList'),  action: () => insertLine('1. ', t('markdown.listItem')) },
  { icon: Link,        title: t('markdown.link'),         action: () => wrap('[', '](url)', t('markdown.linkPlaceholder')) },
])

const viewModes = computed(() => [
  { v: 'edit',    icon: Edit2,   label: t('markdown.editMode') },
  { v: 'split',   icon: Columns, label: t('markdown.splitMode') },
  { v: 'preview', icon: Eye,     label: t('markdown.previewMode') },
])
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden bg-background"
       :class="props.variant === 'flat' ? '' : 'border border-input rounded-lg'">
    <!-- Toolbar -->
    <div class="flex items-center gap-0.5 px-2 py-1.5 border-b border-border bg-muted/30 shrink-0">
      <!-- Format buttons -->
      <template v-for="tool in tools" :key="tool.title">
        <button
          type="button"
          :title="tool.title"
          @click="tool.action"
          class="p-1.5 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
        >
          <component :is="tool.icon" class="w-3.5 h-3.5" />
        </button>
      </template>

      <div class="flex-1" />

      <!-- View mode toggle -->
      <div class="flex items-center rounded-md border border-border overflow-hidden">
        <button
          v-for="{ v, icon, label } in viewModes"
          :key="v"
          type="button"
          :title="label"
          @click="mode = v as ViewMode"
          class="px-2 py-1 text-xs transition-colors"
          :class="mode === v ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted'"
        >
          <component :is="icon" class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Editor / Preview area -->
    <div class="flex flex-1 overflow-hidden min-h-0">
      <!-- Edit pane -->
      <textarea
        v-show="mode === 'edit' || mode === 'split'"
        ref="textareaRef"
        :value="modelValue"
        :placeholder="placeholder || t('markdown.defaultPlaceholder')"
        @input="onInput"
        class="flex-1 resize-none p-4 text-sm font-mono leading-relaxed bg-background text-foreground
               focus:outline-none border-0 min-h-0"
        :class="mode === 'split' ? 'border-r border-border w-1/2' : 'w-full'"
        spellcheck="false"
      />

      <!-- Preview pane -->
      <div
        v-show="mode === 'preview' || mode === 'split'"
        class="flex-1 overflow-y-auto p-4 min-h-0"
        :class="mode === 'split' ? 'w-1/2' : 'w-full'"
      >
        <div
          v-if="modelValue"
          class="prose prose-sm max-w-none
                 prose-headings:font-semibold
                 prose-code:before:content-none prose-code:after:content-none
                 prose-code:bg-muted prose-code:px-1 prose-code:py-0.5 prose-code:rounded prose-code:text-sm"
          v-html="rendered"
        />
        <p v-else class="text-muted-foreground text-sm italic">{{ t('markdown.emptyPreview') }}</p>
      </div>
    </div>
  </div>
</template>
