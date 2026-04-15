<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { Search, X, Bookmark, KeyRound, FileText, BarChart3, Image } from 'lucide-vue-next'
import type { SearchResult, SearchFilters } from '../../types'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const router = useRouter()
const { t } = useI18n()
const query = ref('')
const results = ref<SearchResult[]>([])
const loading = ref(false)
const selectedIndex = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)

const entityTypeIcons: Record<string, typeof Bookmark> = {
  bookmark: Bookmark,
  account: KeyRound,
  note: FileText,
  report: BarChart3,
  media: Image,
}

const entityTypeRoutes: Record<string, string> = {
  bookmark: 'bookmark-detail',
  account: 'account-detail',
  note: 'note-detail',
  report: 'report-detail',
}

watch(
  () => props.visible,
  (val) => {
    if (val) {
      query.value = ''
      results.value = []
      selectedIndex.value = 0
      setTimeout(() => inputRef.value?.focus(), 50)
    }
  }
)

async function handleSearch() {
  if (!query.value.trim()) {
    results.value = []
    return
  }
  loading.value = true
  try {
    const filters: SearchFilters = {}
    results.value = await invoke<SearchResult[]>('search', {
      query: query.value,
      filters,
    })
    selectedIndex.value = 0
  } catch {
    results.value = []
  } finally {
    loading.value = false
  }
}

let debounceTimer: ReturnType<typeof setTimeout> | null = null
function onInput() {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(handleSearch, 300)
}

function selectResult(result: SearchResult) {
  const routeName = entityTypeRoutes[result.entity_type]
  if (routeName) {
    router.push({ name: routeName, params: { id: result.entity_id } })
  }
  emit('close')
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = Math.min(selectedIndex.value + 1, results.value.length - 1)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
  } else if (e.key === 'Enter' && results.value.length) {
    selectResult(results.value[selectedIndex.value])
  } else if (e.key === 'Escape') {
    emit('close')
  }
}

function handleOverlayClick(e: MouseEvent) {
  if ((e.target as HTMLElement).classList.contains('search-overlay')) {
    emit('close')
  }
}

function onKeydownGlobal(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.visible) {
    emit('close')
  }
}

onMounted(() => document.addEventListener('keydown', onKeydownGlobal))
onUnmounted(() => document.removeEventListener('keydown', onKeydownGlobal))
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div
        v-if="visible"
        class="search-overlay fixed inset-0 z-50 bg-foreground/20 backdrop-blur-sm flex items-start justify-center pt-[15vh]"
        @click="handleOverlayClick"
      >
        <div
          class="w-full max-w-lg bg-card border border-border rounded-xl shadow-xl overflow-hidden"
          @click.stop
        >
          <!-- Search input -->
          <div class="flex items-center gap-3 px-4 py-3 border-b border-border">
            <Search class="w-5 h-5 text-muted-foreground shrink-0" />
            <input
              ref="inputRef"
              v-model="query"
              @input="onInput"
              @keydown="handleKeydown"
              :placeholder="t('search.placeholder')"
              class="flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground"
            />
            <button @click="emit('close')" class="text-muted-foreground hover:text-foreground transition-colors">
              <X class="w-4 h-4" />
            </button>
          </div>

          <!-- Results -->
          <div v-if="results.length" class="max-h-80 overflow-y-auto py-1">
            <button
              v-for="(result, i) in results"
              :key="`${result.entity_type}-${result.entity_id}`"
              @click="selectResult(result)"
              class="w-full flex items-center gap-3 px-4 py-2.5 text-sm text-left transition-colors"
              :class="i === selectedIndex ? 'bg-accent' : 'hover:bg-accent/50'"
            >
              <component
                :is="entityTypeIcons[result.entity_type] || FileText"
                class="w-4 h-4 text-muted-foreground shrink-0"
              />
              <div class="flex-1 min-w-0">
                <div class="font-medium truncate">{{ result.title }}</div>
                <div class="text-xs text-muted-foreground truncate">{{ result.content }}</div>
              </div>
              <span class="text-xs text-muted-foreground px-1.5 py-0.5 rounded bg-muted">
                {{ t(`nav.${result.entity_type === 'bookmark' ? 'bookmarks' : result.entity_type === 'account' ? 'accounts' : result.entity_type === 'note' ? 'notes' : result.entity_type === 'report' ? 'reports' : 'media'}`) }}
              </span>
            </button>
          </div>

          <!-- Empty state -->
          <div v-else-if="query && !loading" class="px-4 py-8 text-center text-sm text-muted-foreground">
            {{ t('search.noResults') }}
          </div>

          <!-- Loading -->
          <div v-else-if="loading" class="px-4 py-4 text-center text-sm text-muted-foreground">
            {{ t('search.searching') }}
          </div>

          <!-- Initial state -->
          <div v-else-if="!query" class="px-4 py-4 text-center text-sm text-muted-foreground">
            {{ t('search.typeToSearch') }}
          </div>

          <!-- Footer hint -->
          <div class="px-4 py-2 border-t border-border flex items-center gap-4 text-xs text-muted-foreground">
            <span class="flex items-center gap-1"><kbd class="px-1 py-0.5 rounded bg-muted text-[10px]">↑↓</kbd> {{ t('common.navigate') }}</span>
            <span class="flex items-center gap-1"><kbd class="px-1 py-0.5 rounded bg-muted text-[10px]">Enter</kbd> {{ t('common.open') }}</span>
            <span class="flex items-center gap-1"><kbd class="px-1 py-0.5 rounded bg-muted text-[10px]">Esc</kbd> {{ t('common.close') }}</span>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
