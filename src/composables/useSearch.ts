import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { SearchResult, SearchFilters } from '../types'

export function useSearch() {
  const isOpen = ref(false)
  const query = ref('')
  const results = ref<SearchResult[]>([])
  const loading = ref(false)

  function open() {
    isOpen.value = true
    query.value = ''
    results.value = []
  }

  function close() {
    isOpen.value = false
    query.value = ''
    results.value = []
  }

  async function search(searchQuery: string, filters?: SearchFilters) {
    if (!searchQuery.trim()) {
      results.value = []
      return
    }

    loading.value = true
    try {
      results.value = await invoke<SearchResult[]>('search', {
        query: searchQuery,
        filters,
      })
    } catch {
      results.value = []
    } finally {
      loading.value = false
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    // Ctrl+K or Cmd+K to open search
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      e.preventDefault()
      if (isOpen.value) {
        close()
      } else {
        open()
      }
    }
    // Escape to close
    if (e.key === 'Escape' && isOpen.value) {
      close()
    }
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeydown)
  })

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown)
  })

  return {
    isOpen,
    query,
    results,
    loading,
    open,
    close,
    search,
  }
}