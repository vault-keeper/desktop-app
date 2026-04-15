<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useWorkspaceStore } from '../stores/workspace'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import {
  Bookmark, Key, FileText, CalendarDays, Image, Tag,
  Plus, ArrowRight, Clock
} from 'lucide-vue-next'

const router = useRouter()
const workspaceStore = useWorkspaceStore()
const { t } = useI18n()

interface Stats {
  bookmarks: number
  accounts: number
  notes: number
  reports: number
  media: number
  tags: number
}

interface RecentItem {
  id: string
  type: string
  title: string
  updated_at: string
}

const stats = ref<Stats>({ bookmarks: 0, accounts: 0, notes: 0, reports: 0, media: 0, tags: 0 })
const recentItems = ref<RecentItem[]>([])
const loading = ref(true)

const statCards = computed(() => [
  { key: 'bookmarks', label: t('nav.bookmarks'), icon: Bookmark, color: 'text-blue-500 bg-blue-500/10', route: 'bookmarks' },
  { key: 'accounts', label: t('nav.accounts'), icon: Key, color: 'text-emerald-500 bg-emerald-500/10', route: 'accounts' },
  { key: 'notes', label: t('nav.notes'), icon: FileText, color: 'text-amber-500 bg-amber-500/10', route: 'notes' },
  { key: 'reports', label: t('nav.reports'), icon: CalendarDays, color: 'text-purple-500 bg-purple-500/10', route: 'reports' },
  { key: 'media', label: t('home.media'), icon: Image, color: 'text-pink-500 bg-pink-500/10', route: 'media' },
  { key: 'tags', label: t('nav.tags'), icon: Tag, color: 'text-teal-500 bg-teal-500/10', route: 'tags' },
])

onMounted(async () => {
  try {
    const wsId = workspaceStore.currentWorkspace?.id
    if (!wsId) return

    const [bks, accts, notes, rpts, media, tags] = await Promise.all([
      invoke<any[]>('list_bookmarks', { groupId: null }),
      invoke<any[]>('list_accounts', { groupId: null }),
      invoke<any[]>('list_notes', { groupId: null }),
      invoke<any[]>('list_reports', { type: '' }),
      invoke<any[]>('list_media_assets'),
      invoke<any[]>('list_tags'),
    ])

    stats.value = {
      bookmarks: bks?.length || 0,
      accounts: accts?.length || 0,
      notes: notes?.length || 0,
      reports: rpts?.length || 0,
      media: media?.length || 0,
      tags: tags?.length || 0,
    }

    const all: RecentItem[] = [
      ...(bks || []).map((b: any) => ({ id: b.id, type: 'bookmark', title: b.title, updated_at: b.updated_at })),
      ...(accts || []).map((a: any) => ({ id: a.id, type: 'account', title: a.title, updated_at: a.updated_at })),
      ...(notes || []).map((n: any) => ({ id: n.id, type: 'note', title: n.title, updated_at: n.updated_at })),
      ...(rpts || []).map((r: any) => ({ id: r.id, type: 'report', title: r.title, updated_at: r.updated_at })),
    ]
    all.sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())
    recentItems.value = all.slice(0, 8)
  } catch {
    // Workspace might not be set up yet
  } finally {
    loading.value = false
  }
})

function goTo(route: string) {
  const wsId = workspaceStore.currentWorkspace?.id
  router.push(`/w/${wsId}/${route}`)
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const days = Math.floor(diff / 86400000)
  if (days === 0) return t('home.today')
  if (days === 1) return t('home.yesterday')
  if (days < 7) return `${days} ${t('home.daysAgo')}`
  return date.toLocaleDateString()
}

function getTypeIcon(type: string) {
  switch (type) {
    case 'bookmark': return Bookmark
    case 'account': return Key
    case 'note': return FileText
    case 'report': return CalendarDays
    default: return FileText
  }
}
</script>

<template>
  <div class="max-w-5xl mx-auto">
    <h1 class="text-2xl font-bold mb-6">{{ t('home.title') }}</h1>

    <!-- Stats Grid -->
    <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-4 mb-8">
      <div
        v-for="card in statCards"
        :key="card.key"
        @click="goTo(card.route)"
        class="bg-card rounded-xl border border-border p-4 cursor-pointer
               hover:border-primary/50 hover:shadow-md transition-all group"
      >
        <div class="flex items-center gap-3 mb-3">
          <div class="w-9 h-9 rounded-lg flex items-center justify-center" :class="card.color">
            <component :is="card.icon" class="w-4 h-4" />
          </div>
        </div>
        <div class="text-2xl font-bold">{{ (stats as any)[card.key] || 0 }}</div>
        <div class="text-sm text-muted-foreground">{{ card.label }}</div>
        <ArrowRight class="w-4 h-4 text-muted-foreground mt-2 opacity-0 group-hover:opacity-100 transition-opacity" />
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="bg-card rounded-xl border border-border p-6 mb-6">
      <h2 class="font-semibold mb-4">{{ t('home.quickActions') }}</h2>
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
        <button
          @click="goTo('bookmarks')"
          class="flex items-center gap-2 px-4 py-3 rounded-lg border border-border
                 hover:bg-muted hover:border-primary/30 transition-all text-sm"
        >
          <Plus class="w-4 h-4" />
          {{ t('home.addBookmark') }}
        </button>
        <button
          @click="goTo('accounts')"
          class="flex items-center gap-2 px-4 py-3 rounded-lg border border-border
                 hover:bg-muted hover:border-primary/30 transition-all text-sm"
        >
          <Plus class="w-4 h-4" />
          {{ t('home.addAccount') }}
        </button>
        <button
          @click="goTo('notes')"
          class="flex items-center gap-2 px-4 py-3 rounded-lg border border-border
                 hover:bg-muted hover:border-primary/30 transition-all text-sm"
        >
          <Plus class="w-4 h-4" />
          {{ t('home.newNote') }}
        </button>
        <button
          @click="goTo('reports')"
          class="flex items-center gap-2 px-4 py-3 rounded-lg border border-border
                 hover:bg-muted hover:border-primary/30 transition-all text-sm"
        >
          <Plus class="w-4 h-4" />
          {{ t('home.writeReport') }}
        </button>
      </div>
    </div>

    <!-- Recent Activity -->
    <div class="bg-card rounded-xl border border-border p-6">
      <h2 class="font-semibold mb-4 flex items-center gap-2">
        <Clock class="w-4 h-4 text-muted-foreground" />
        {{ t('home.recentActivity') }}
      </h2>

      <div v-if="loading" class="space-y-3">
        <div v-for="i in 4" :key="i" class="h-12 bg-muted/50 rounded-lg animate-pulse" />
      </div>

      <div v-else-if="recentItems.length === 0" class="text-center py-8 text-muted-foreground">
        <p>{{ t('home.noContent') }}</p>
        <p class="text-sm mt-1">{{ t('home.noContentPrompt') }}</p>
      </div>

      <div v-else class="space-y-2">
        <div
          v-for="item in recentItems"
          :key="item.id + item.type"
          @click="goTo(item.type + 's')"
          class="flex items-center gap-3 p-3 rounded-lg hover:bg-muted transition-colors cursor-pointer"
        >
          <component :is="getTypeIcon(item.type)" class="w-4 h-4 text-muted-foreground shrink-0" />
          <span class="flex-1 text-sm truncate">{{ item.title }}</span>
          <span class="text-xs text-muted-foreground shrink-0">{{ formatDate(item.updated_at) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
