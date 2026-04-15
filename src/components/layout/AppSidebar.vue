<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '../../stores/auth'
import { useWorkspaceStore } from '../../stores/workspace'
import { useI18n } from 'vue-i18n'
import {
  Bookmark,
  KeyRound,
  FileText,
  BarChart3,
  Image,
  Tags,
  Settings,
  Lock,
  ChevronLeft,
  ChevronRight,
  Home,
} from 'lucide-vue-next'

const props = defineProps<{
  collapsed?: boolean
}>()

const emit = defineEmits<{
  (e: 'toggle-collapse'): void
}>()

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()
const workspaceStore = useWorkspaceStore()
const { t } = useI18n()

const navItems = computed(() => [
  { name: 'workspace-home', label: t('nav.overview'), icon: Home },
  { name: 'bookmarks', label: t('nav.bookmarks'), icon: Bookmark },
  { name: 'accounts', label: t('nav.accounts'), icon: KeyRound },
  { name: 'notes', label: t('nav.notes'), icon: FileText },
  { name: 'reports', label: t('nav.reports'), icon: BarChart3 },
  { name: 'media', label: t('nav.media'), icon: Image },
  { name: 'tags', label: t('nav.tags'), icon: Tags },
  { name: 'settings', label: t('nav.settings'), icon: Settings },
])

function isActive(name: string) {
  if (name === 'workspace-home') {
    return route.name === 'workspace-home'
  }
  return route.path.includes('/' + name)
}

async function handleLock() {
  await authStore.lockVault()
  router.push({ name: 'lock' })
}

function navigate(name: string) {
  router.push({ name })
}
</script>

<template>
  <aside
    class="h-full flex flex-col border-r border-border bg-card transition-all duration-300"
    :class="collapsed ? 'w-16' : 'w-56'"
  >
    <!-- Workspace header -->
    <div class="px-3 py-4 border-b border-border">
      <div class="flex items-center gap-2" :class="collapsed ? 'justify-center' : ''">
        <div
          v-if="workspaceStore.currentWorkspace"
          class="w-8 h-8 rounded-lg flex items-center justify-center text-sm font-bold shrink-0"
          :style="{ backgroundColor: workspaceStore.currentWorkspace.color || 'hsl(var(--primary))', color: 'hsl(var(--primary-foreground))' }"
        >
          {{ workspaceStore.currentWorkspace.icon || workspaceStore.currentWorkspace.name.charAt(0).toUpperCase() }}
        </div>
        <div v-else class="w-8 h-8 rounded-lg bg-primary flex items-center justify-center text-primary-foreground text-sm font-bold shrink-0">
          V
        </div>
        <div v-if="!collapsed" class="min-w-0 flex-1">
          <div class="text-sm font-semibold truncate">
            {{ workspaceStore.currentWorkspace?.name || 'VaultKeeper' }}
          </div>
        </div>
      </div>
    </div>

    <!-- Navigation -->
    <nav class="flex-1 px-2 py-3 space-y-0.5 overflow-y-auto">
      <button
        v-for="item in navItems"
        :key="item.name"
        @click="navigate(item.name)"
        class="w-full flex items-center gap-2.5 px-2.5 py-2 rounded-lg text-sm transition-colors"
        :class="[
          isActive(item.name)
            ? 'bg-primary text-primary-foreground'
            : 'text-muted-foreground hover:bg-accent hover:text-foreground',
          collapsed ? 'justify-center' : ''
        ]"
        :title="collapsed ? item.label : undefined"
      >
        <component :is="item.icon" class="w-4 h-4 shrink-0" />
        <span v-if="!collapsed" class="truncate">{{ item.label }}</span>
      </button>
    </nav>

    <!-- Bottom actions -->
    <div class="px-2 py-3 border-t border-border space-y-0.5">
      <button
        @click="handleLock"
        class="w-full flex items-center gap-2.5 px-2.5 py-2 rounded-lg text-sm text-muted-foreground hover:bg-accent hover:text-foreground transition-colors"
        :class="collapsed ? 'justify-center' : ''"
        :title="collapsed ? t('nav.lock') : undefined"
      >
        <Lock class="w-4 h-4 shrink-0" />
        <span v-if="!collapsed">{{ t('nav.lock') }}</span>
      </button>

      <button
        @click="emit('toggle-collapse')"
        class="w-full flex items-center gap-2.5 px-2.5 py-2 rounded-lg text-sm text-muted-foreground hover:bg-accent hover:text-foreground transition-colors"
        :class="collapsed ? 'justify-center' : ''"
        :title="collapsed ? t('nav.expandSidebar') : t('nav.collapseSidebar')"
      >
        <ChevronRight v-if="collapsed" class="w-4 h-4 shrink-0" />
        <ChevronLeft v-else class="w-4 h-4 shrink-0" />
        <span v-if="!collapsed">{{ t('nav.collapseSidebar') }}</span>
      </button>
    </div>
  </aside>
</template>
