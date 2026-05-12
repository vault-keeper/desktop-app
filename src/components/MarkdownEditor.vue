<script setup lang="ts">
import { ref, computed, onBeforeUnmount, nextTick, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { renderMarkdown } from '../lib/markdown'
import '../styles/hljs-theme.css'
import {
  Bold, Italic, Strikethrough,
  Heading1, Heading2, Heading3,
  Code, Code2,
  Link as LinkIcon, Image as ImageIcon,
  List, ListOrdered, ListChecks,
  Quote, Minus, Table as TableIcon,
  Eye, Edit2, Columns,
  Maximize2, Minimize2,
} from 'lucide-vue-next'

const props = defineProps<{
  modelValue: string
  placeholder?: string
  variant?: 'default' | 'flat'
}>()
const emit = defineEmits<{ (e: 'update:modelValue', v: string): void }>()

const { t } = useI18n()

type ViewMode = 'edit' | 'preview' | 'split'
const mode = ref<ViewMode>('split')
const fullscreen = ref(false)
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const previewRef = ref<HTMLDivElement | null>(null)

const rendered = computed(() => renderMarkdown(props.modelValue))

// --- Stats --------------------------------------------------------------

const stats = computed(() => {
  const text = props.modelValue || ''
  const chars = text.length
  const words = text.trim() ? text.trim().split(/\s+/).length : 0
  const lines = text ? text.split('\n').length : 1
  return { chars, words, lines }
})

// --- Editing helpers ----------------------------------------------------

function setValue(newVal: string, selStart: number, selEnd: number) {
  emit('update:modelValue', newVal)
  nextTick(() => {
    const ta = textareaRef.value
    if (!ta) return
    ta.focus()
    ta.setSelectionRange(selStart, selEnd)
  })
}

function onInput(e: Event) {
  emit('update:modelValue', (e.target as HTMLTextAreaElement).value)
}

/** Wrap selection (or insert placeholder) with `before`/`after`. */
function wrap(before: string, after: string, placeholder: string) {
  const ta = textareaRef.value
  if (!ta) return
  const start = ta.selectionStart
  const end = ta.selectionEnd
  const selected = ta.value.slice(start, end) || placeholder
  const newVal = ta.value.slice(0, start) + before + selected + after + ta.value.slice(end)
  setValue(newVal, start + before.length, start + before.length + selected.length)
}

/** Insert a prefix at the start of the current line. */
function insertLine(prefix: string, placeholder = '') {
  const ta = textareaRef.value
  if (!ta) return
  const start = ta.selectionStart
  const end = ta.selectionEnd
  const lineStart = ta.value.lastIndexOf('\n', start - 1) + 1
  // Avoid adding the prefix twice if it's already there
  const head = ta.value.slice(lineStart, lineStart + prefix.length)
  if (head === prefix) {
    const newStart = start
    const newEnd = end
    setValue(ta.value, newStart, newEnd)
    return
  }
  const newVal = ta.value.slice(0, lineStart) + prefix + ta.value.slice(lineStart)
  const offset = prefix.length
  setValue(newVal, start + offset, end + offset || lineStart + prefix.length + placeholder.length)
}

/** Insert a multi-line block at the cursor (on its own paragraph). */
function insertBlock(block: string, cursorOffset?: number) {
  const ta = textareaRef.value
  if (!ta) return
  const start = ta.selectionStart
  const end = ta.selectionEnd
  const before = ta.value.slice(0, start)
  const after = ta.value.slice(end)
  // Make sure the block sits on its own line
  const needLeadingNl = before.length > 0 && !before.endsWith('\n') ? '\n' : ''
  const needTrailingNl = after.length > 0 && !after.startsWith('\n') ? '\n' : ''
  const inserted = needLeadingNl + block + needTrailingNl
  const newVal = before + inserted + after
  const cursor = start + needLeadingNl.length + (cursorOffset ?? block.length)
  setValue(newVal, cursor, cursor)
}

// --- Toolbar actions ----------------------------------------------------

function actBold()       { wrap('**', '**', t('markdown.boldPlaceholder')) }
function actItalic()     { wrap('*', '*', t('markdown.italicPlaceholder')) }
function actStrike()     { wrap('~~', '~~', t('markdown.strikePlaceholder')) }
function actInlineCode() { wrap('`', '`', t('markdown.codePlaceholder')) }
function actLink()       { wrap('[', '](https://)', t('markdown.linkPlaceholder')) }
function actImage()      { wrap('![', '](https://)', t('markdown.imagePlaceholder')) }
function actH1()         { insertLine('# ',   t('markdown.headingPlaceholder')) }
function actH2()         { insertLine('## ',  t('markdown.headingPlaceholder')) }
function actH3()         { insertLine('### ', t('markdown.headingPlaceholder')) }
function actQuote()      { insertLine('> ',   t('markdown.quotePlaceholder')) }
function actBullet()     { insertLine('- ',   t('markdown.listItem')) }
function actOrdered()    { insertLine('1. ',  t('markdown.listItem')) }
function actTask()       { insertLine('- [ ] ', t('markdown.taskItem')) }
function actHr()         { insertBlock('\n---\n') }
function actCodeBlock()  {
  const ta = textareaRef.value
  if (!ta) return
  const sel = ta.value.slice(ta.selectionStart, ta.selectionEnd)
  if (sel) {
    wrap('```\n', '\n```', sel)
  } else {
    insertBlock('```\n' + t('markdown.codePlaceholder') + '\n```', 4)
  }
}
function actTable() {
  const block =
    '| ' + t('markdown.tableHeader') + ' 1 | ' + t('markdown.tableHeader') + ' 2 |\n' +
    '| --- | --- |\n' +
    '| ' + t('markdown.tableCell') + ' | ' + t('markdown.tableCell') + ' |'
  insertBlock(block)
}
function toggleFullscreen() { fullscreen.value = !fullscreen.value }

// --- Keyboard handling --------------------------------------------------

const LIST_RE = /^(\s*)(?:([-*+])\s+(\[[ xX]\]\s+)?|(\d+)\.\s+)/

function onKeydown(e: KeyboardEvent) {
  // Modifier-based shortcuts (Ctrl on Win/Linux, Cmd on Mac)
  if ((e.ctrlKey || e.metaKey) && !e.altKey) {
    const k = e.key.toLowerCase()
    if (!e.shiftKey) {
      if (k === 'b')  { e.preventDefault(); return actBold() }
      if (k === 'i')  { e.preventDefault(); return actItalic() }
      if (k === 'k')  { e.preventDefault(); return actLink() }
      if (k === 'e')  { e.preventDefault(); return actInlineCode() }
      if (k === '1')  { e.preventDefault(); return actH1() }
      if (k === '2')  { e.preventDefault(); return actH2() }
      if (k === '3')  { e.preventDefault(); return actH3() }
      if (k === 'd')  { e.preventDefault(); return actStrike() }
    } else {
      if (k === 'k')  { e.preventDefault(); return actCodeBlock() }
    }
  }

  if (e.key === 'F11') {
    e.preventDefault()
    toggleFullscreen()
    return
  }

  if (e.key === 'Escape' && fullscreen.value) {
    e.preventDefault()
    fullscreen.value = false
    return
  }

  if (e.key === 'Tab') {
    e.preventDefault()
    const ta = textareaRef.value
    if (!ta) return
    const start = ta.selectionStart
    const end = ta.selectionEnd
    if (e.shiftKey) {
      // De-indent the current line(s) by up to 2 spaces.
      const lineStart = ta.value.lastIndexOf('\n', start - 1) + 1
      const head = ta.value.slice(lineStart, lineStart + 2)
      const strip = head.startsWith('  ') ? 2 : head.startsWith(' ') ? 1 : 0
      if (strip > 0) {
        const newVal = ta.value.slice(0, lineStart) + ta.value.slice(lineStart + strip)
        setValue(newVal, Math.max(lineStart, start - strip), Math.max(lineStart, end - strip))
      }
    } else {
      const newVal = ta.value.slice(0, start) + '  ' + ta.value.slice(end)
      setValue(newVal, start + 2, start + 2)
    }
    return
  }

  if (e.key === 'Enter' && !e.shiftKey && !e.ctrlKey && !e.metaKey && !e.altKey) {
    const ta = textareaRef.value
    if (!ta) return
    const start = ta.selectionStart
    if (start !== ta.selectionEnd) return  // selection — fall through
    const lineStart = ta.value.lastIndexOf('\n', start - 1) + 1
    const line = ta.value.slice(lineStart, start)
    const m = line.match(LIST_RE)
    if (!m) return

    const indent = m[1] ?? ''
    const bullet = m[2]      // - * +
    const taskBox = m[3]     // [ ] / [x] (optional)
    const ordNum = m[4]      // ordered marker

    // Pressing Enter on an empty list item exits the list
    const restAfterMarker = line.slice(m[0].length)
    if (!restAfterMarker.trim()) {
      const before = ta.value.slice(0, lineStart)
      const after = ta.value.slice(start)
      const newVal = before + indent + after
      e.preventDefault()
      setValue(newVal, lineStart + indent.length, lineStart + indent.length)
      return
    }

    let nextMarker = ''
    if (bullet) {
      nextMarker = `${bullet} ${taskBox ? '[ ] ' : ''}`
    } else if (ordNum) {
      nextMarker = `${parseInt(ordNum, 10) + 1}. `
    }
    const insertion = '\n' + indent + nextMarker
    const newVal = ta.value.slice(0, start) + insertion + ta.value.slice(ta.selectionEnd)
    e.preventDefault()
    const pos = start + insertion.length
    setValue(newVal, pos, pos)
  }
}

// --- Synchronized scroll (split mode) -----------------------------------

let syncing = false
function onEditorScroll() {
  if (mode.value !== 'split' || syncing) return
  const ta = textareaRef.value
  const pv = previewRef.value
  if (!ta || !pv) return
  syncing = true
  const denom = Math.max(1, ta.scrollHeight - ta.clientHeight)
  const ratio = ta.scrollTop / denom
  pv.scrollTop = ratio * Math.max(0, pv.scrollHeight - pv.clientHeight)
  requestAnimationFrame(() => { syncing = false })
}

function onPreviewScroll() {
  if (mode.value !== 'split' || syncing) return
  const ta = textareaRef.value
  const pv = previewRef.value
  if (!ta || !pv) return
  syncing = true
  const denom = Math.max(1, pv.scrollHeight - pv.clientHeight)
  const ratio = pv.scrollTop / denom
  ta.scrollTop = ratio * Math.max(0, ta.scrollHeight - ta.clientHeight)
  requestAnimationFrame(() => { syncing = false })
}

// --- Body scroll lock while fullscreen ----------------------------------

watch(fullscreen, (v) => {
  if (typeof document === 'undefined') return
  document.body.style.overflow = v ? 'hidden' : ''
})

onBeforeUnmount(() => {
  if (typeof document !== 'undefined') document.body.style.overflow = ''
})

// --- Toolbar config -----------------------------------------------------

const toolGroups = computed(() => [
  [
    { icon: Heading1,      title: t('markdown.heading1'),     action: actH1 },
    { icon: Heading2,      title: t('markdown.heading2'),     action: actH2 },
    { icon: Heading3,      title: t('markdown.heading3'),     action: actH3 },
  ],
  [
    { icon: Bold,          title: t('markdown.bold'),         action: actBold },
    { icon: Italic,        title: t('markdown.italic'),       action: actItalic },
    { icon: Strikethrough, title: t('markdown.strike'),       action: actStrike },
  ],
  [
    { icon: Code,          title: t('markdown.code'),         action: actInlineCode },
    { icon: Code2,         title: t('markdown.codeBlock'),    action: actCodeBlock },
  ],
  [
    { icon: List,          title: t('markdown.bulletList'),   action: actBullet },
    { icon: ListOrdered,   title: t('markdown.orderedList'),  action: actOrdered },
    { icon: ListChecks,    title: t('markdown.taskList'),     action: actTask },
  ],
  [
    { icon: Quote,         title: t('markdown.quote'),        action: actQuote },
    { icon: LinkIcon,      title: t('markdown.link'),         action: actLink },
    { icon: ImageIcon,     title: t('markdown.image'),        action: actImage },
    { icon: TableIcon,     title: t('markdown.table'),        action: actTable },
    { icon: Minus,         title: t('markdown.hr'),           action: actHr },
  ],
])

const viewModes = computed(() => [
  { v: 'edit',    icon: Edit2,   label: t('markdown.editMode') },
  { v: 'split',   icon: Columns, label: t('markdown.splitMode') },
  { v: 'preview', icon: Eye,     label: t('markdown.previewMode') },
])
</script>

<template>
  <Teleport to="body" :disabled="!fullscreen">
    <div
      class="flex flex-col overflow-hidden bg-background"
      :class="[
        fullscreen
          ? 'fixed inset-0 z-50 h-screen w-screen'
          : 'h-full ' + (props.variant === 'flat' ? '' : 'border border-input rounded-lg'),
      ]"
    >
      <!-- Toolbar -->
      <div class="flex items-center flex-wrap gap-0.5 px-2 py-1.5 border-b border-border bg-muted/30 shrink-0">
        <template v-for="(group, gi) in toolGroups" :key="gi">
          <button
            v-for="tool in group"
            :key="tool.title"
            type="button"
            :title="tool.title"
            @click="tool.action"
            class="p-1.5 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
          >
            <component :is="tool.icon" class="w-3.5 h-3.5" />
          </button>
          <span v-if="gi < toolGroups.length - 1" class="mx-1 h-4 w-px bg-border" />
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

        <button
          type="button"
          :title="fullscreen ? t('markdown.exitFullscreen') : t('markdown.fullscreen')"
          @click="toggleFullscreen"
          class="ml-1 p-1.5 rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
        >
          <component :is="fullscreen ? Minimize2 : Maximize2" class="w-3.5 h-3.5" />
        </button>
      </div>

      <!-- Editor / Preview area -->
      <div class="flex flex-1 overflow-hidden min-h-0">
        <textarea
          v-show="mode === 'edit' || mode === 'split'"
          ref="textareaRef"
          :value="modelValue"
          :placeholder="placeholder || t('markdown.defaultPlaceholder')"
          @input="onInput"
          @keydown="onKeydown"
          @scroll="onEditorScroll"
          class="flex-1 resize-none p-4 text-sm font-mono leading-relaxed bg-background text-foreground
                 focus:outline-none border-0 min-h-0 overflow-y-auto"
          :class="mode === 'split' ? 'border-r border-border w-1/2' : 'w-full'"
          spellcheck="false"
        />

        <div
          v-show="mode === 'preview' || mode === 'split'"
          ref="previewRef"
          @scroll="onPreviewScroll"
          class="flex-1 overflow-y-auto p-4 min-h-0"
          :class="mode === 'split' ? 'w-1/2' : 'w-full'"
        >
          <div
            v-if="modelValue"
            class="prose prose-sm max-w-none
                   prose-headings:font-semibold
                   prose-code:before:content-none prose-code:after:content-none
                   prose-code:bg-muted prose-code:px-1 prose-code:py-0.5 prose-code:rounded prose-code:text-sm
                   prose-pre:bg-transparent prose-pre:p-0
                   prose-table:border prose-table:border-border
                   prose-th:border prose-th:border-border prose-th:px-3 prose-th:py-1.5 prose-th:bg-muted/50
                   prose-td:border prose-td:border-border prose-td:px-3 prose-td:py-1.5
                   prose-img:rounded-md prose-img:border prose-img:border-border"
            v-html="rendered"
          />
          <p v-else class="text-muted-foreground text-sm italic">{{ t('markdown.emptyPreview') }}</p>
        </div>
      </div>

      <!-- Status bar -->
      <div class="flex items-center justify-between gap-3 px-3 py-1 text-[11px] text-muted-foreground
                  border-t border-border bg-muted/20 shrink-0 select-none">
        <div class="flex items-center gap-3">
          <span>{{ t('markdown.linesLabel') }}: {{ stats.lines }}</span>
          <span>{{ t('markdown.wordsLabel') }}: {{ stats.words }}</span>
          <span>{{ t('markdown.charsLabel') }}: {{ stats.chars }}</span>
        </div>
        <div class="opacity-70">
          {{ fullscreen ? t('markdown.fullscreenHint') : t('markdown.shortcutHint') }}
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
:deep(.task-list-item) {
  list-style: none;
  margin-left: -1.25rem;
}
:deep(.task-list-checkbox) {
  margin-right: 0.4rem;
  vertical-align: middle;
}
</style>
