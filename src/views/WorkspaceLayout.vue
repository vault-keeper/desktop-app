<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useWorkspaceStore } from '../stores/workspace'
import { useAuthStore } from '../stores/auth'
import {
  Bookmark, Key, FileText, CalendarDays, Image,
  Tag, Settings, Lock, Menu
} from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()
const workspaceStore = useWorkspaceStore()
const authStore = useAuthStore()
const { t } = useI18n()

const sidebarCollapsed = ref(false)

const navItems = computed(() => [
  { name: t('home.title'), icon: FileText, route: 'workspace-home' },
  { name: t('nav.bookmarks'), icon: Bookmark, route: 'bookmarks' },
  { name: t('nav.accounts'), icon: Key, route: 'accounts' },
  { name: t('nav.notes'), icon: FileText, route: 'notes' },
  { name: t('nav.reports'), icon: CalendarDays, route: 'reports' },
  { name: t('home.media'), icon: Image, route: 'media' },
  { name: t('nav.tags'), icon: Tag, route: 'tags' },
])

function isActive(routeName: string): boolean {
  const currentRoute = route.name?.toString() || ''
  return currentRoute.startsWith(routeName) || currentRoute === routeName
}

async function handleLock() {
  await authStore.lockVault()
  workspaceStore.clearCurrentWorkspace()
  router.push({ name: 'lock' })
}

function toggleSidebar() {
  sidebarCollapsed.value = !sidebarCollapsed.value
}

const ws = workspaceStore.currentWorkspace
</script>

<template>
  <div class="flex h-screen bg-background overflow-hidden">
    <!-- Sidebar -->
    <aside
      class="flex flex-col bg-card border-r border-border transition-all duration-300"
      :class="sidebarCollapsed ? 'w-16' : 'w-60'"
    >
      <!-- Workspace header -->
      <div class="p-3 border-b border-border">
        <div class="flex items-center gap-3">
          <button
            @click="toggleSidebar"
            class="p-1.5 rounded-md hover:bg-muted transition-colors text-muted-foreground"
          >
            <Menu class="w-4 h-4" />
          </button>
          <template v-if="!sidebarCollapsed">
            <span class="text-lg">{{ ws?.icon || '📁' }}</span>
            <span class="font-semibold text-sm truncate">{{ ws?.name }}</span>
          </template>
        </div>
      </div>

      <!-- Navigation -->
      <nav class="flex-1 py-2 overflow-y-auto">
        <router-link
          v-for="item in navItems"
          :key="item.route"
          :to="{ name: item.route, params: { workspaceId: ws?.id } }"
          class="flex items-center gap-3 px-3 py-2.5 mx-2 rounded-lg text-sm transition-colors"
          :class="[
            isActive(item.route)
              ? 'bg-primary/10 text-primary font-medium'
              : 'text-muted-foreground hover:bg-muted hover:text-foreground'
          ]"
          :title="sidebarCollapsed ? item.name : undefined"
        >
          <component :is="item.icon" class="w-4 h-4 shrink-0" />
          <span v-if="!sidebarCollapsed">{{ item.name }}</span>
        </router-link>
      </nav>

      <!-- Footer -->
      <div class="p-2 border-t border-border space-y-1">
        <router-link
          :to="{ name: 'settings', params: { workspaceId: ws?.id } }"
          class="flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-muted-foreground
                 hover:bg-muted hover:text-foreground transition-colors"
          :title="sidebarCollapsed ? t('nav.settings') : undefined"
        >
          <Settings class="w-4 h-4 shrink-0" />
          <span v-if="!sidebarCollapsed">{{ t('nav.settings') }}</span>
        </router-link>
        <button
          @click="handleLock"
          class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm text-muted-foreground
                 hover:bg-destructive/10 hover:text-destructive transition-colors"
          :title="sidebarCollapsed ? t('nav.lock') : undefined"
        >
          <Lock class="w-4 h-4 shrink-0" />
          <span v-if="!sidebarCollapsed">{{ t('nav.lock') }}</span>
        </button>
      </div>
    </aside>

    <!-- Main content -->
    <main class="flex-1 overflow-hidden flex flex-col">
      <!-- Top bar -->
      <header class="h-14 border-b border-border bg-card/50 px-6 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-2 text-sm text-muted-foreground">
          <span>{{ ws?.name }}</span>
        </div>
        <div class="flex items-center gap-2">
          <button
            @click="router.push({ name: 'workspaces' })"
            class="text-sm text-muted-foreground hover:text-foreground transition-colors"
          >
            {{ t('nav.switchWorkspace') }}
          </button>
        </div>
      </header>

      <!-- Page content -->
      <div class="flex-1 overflow-y-auto p-6">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </div>
    </main>
  </div>
</template>
