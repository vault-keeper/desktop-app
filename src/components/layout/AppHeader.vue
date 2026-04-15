<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Search, Moon, Sun } from 'lucide-vue-next'

const props = defineProps<{
  isDark: boolean
}>()

const emit = defineEmits<{
  (e: 'search'): void
  (e: 'toggle-theme'): void
}>()

const route = useRoute()
const { t } = useI18n()

const pageTitle = computed(() => {
  const nameMap: Record<string, string> = {
    'workspace-home': t('nav.overview'),
    bookmarks: t('nav.bookmarks'),
    'bookmark-detail': t('header.bookmarkDetail'),
    accounts: t('nav.accounts'),
    'account-detail': t('header.accountDetail'),
    notes: t('nav.notes'),
    'note-detail': t('header.noteDetail'),
    reports: t('nav.reports'),
    'report-detail': t('header.reportDetail'),
    media: t('nav.media'),
    tags: t('nav.tags'),
    settings: t('nav.settings'),
  }
  return nameMap[route.name as string] || 'VaultKeeper'
})

function onKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
    e.preventDefault()
    emit('search')
  }
}

onMounted(() => document.addEventListener('keydown', onKeydown))
onUnmounted(() => document.removeEventListener('keydown', onKeydown))
</script>

<template>
  <header class="h-14 flex items-center justify-between px-4 border-b border-border bg-card/80 backdrop-blur-sm sticky top-0 z-30">
    <!-- Left: Page title -->
    <h1 class="text-lg font-semibold">{{ pageTitle }}</h1>

    <!-- Right: Actions -->
    <div class="flex items-center gap-2">
      <!-- Search trigger -->
      <button
        @click="emit('search')"
        class="flex items-center gap-2 px-3 py-1.5 rounded-lg border border-border text-sm text-muted-foreground hover:bg-accent hover:text-foreground transition-colors"
      >
        <Search class="w-4 h-4" />
        <span class="hidden sm:inline">{{ t('header.search') }}</span>
        <kbd class="hidden sm:inline text-[10px] px-1 py-0.5 rounded bg-muted">⌘K</kbd>
      </button>

      <!-- Theme toggle -->
      <button
        @click="emit('toggle-theme')"
        class="p-2 rounded-lg border border-border text-muted-foreground hover:bg-accent hover:text-foreground transition-colors"
        :title="isDark ? t('header.toggleLight') : t('header.toggleDark')"
      >
        <Moon v-if="!isDark" class="w-4 h-4" />
        <Sun v-else class="w-4 h-4" />
      </button>
    </div>
  </header>
</template>
