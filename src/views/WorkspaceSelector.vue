<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useWorkspaceStore } from '../stores/workspace'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import { useI18n } from 'vue-i18n'
import {
  Plus, Settings, Lock, FolderOpen, Trash2,
  Edit3, Check, X, LayoutGrid
} from 'lucide-vue-next'

const workspaceStore = useWorkspaceStore()
const authStore = useAuthStore()
const router = useRouter()
const { t } = useI18n()

const showCreateForm = ref(false)
const newWorkspaceName = ref('')
const newWorkspaceIcon = ref('📁')
const newWorkspaceColor = ref('#6366f1')
const editingId = ref<string | null>(null)
const editName = ref('')
const pendingDeleteId = ref<string | null>(null)

const pendingDeleteWorkspace = computed(() =>
  workspaceStore.workspaces.find(w => w.id === pendingDeleteId.value) ?? null
)

const colorOptions = [
  '#6366f1', '#8b5cf6', '#ec4899', '#ef4444',
  '#f97316', '#eab308', '#22c55e', '#14b8a6',
  '#06b6d4', '#3b82f6'
]

const iconOptions = [
  '📁', '💼', '🏠', '🚀', '⚡', '🎯', '💡', '🔐',
  '📊', '🎨', '📝', '🌐', '📱', '💻', '☁️', '🛠️'
]

onMounted(async () => {
  await workspaceStore.fetchWorkspaces()
})

async function createWorkspace() {
  if (!newWorkspaceName.value.trim()) return
  const ws = await workspaceStore.createWorkspace(
    newWorkspaceName.value.trim(),
    newWorkspaceIcon.value,
    newWorkspaceColor.value
  )
  showCreateForm.value = false
  newWorkspaceName.value = ''
  await workspaceStore.switchWorkspace(ws.id)
  router.push(`/w/${ws.id}`)
}

async function selectWorkspace(id: string) {
  await workspaceStore.switchWorkspace(id)
  router.push(`/w/${id}`)
}

async function deleteWorkspace(id: string, e: Event) {
  e.stopPropagation()
  pendingDeleteId.value = id
}

async function confirmDelete() {
  if (!pendingDeleteId.value) return
  await workspaceStore.deleteWorkspace(pendingDeleteId.value)
  pendingDeleteId.value = null
}

function startEdit(id: string, name: string, e: Event) {
  e.stopPropagation()
  editingId.value = id
  editName.value = name
}

function saveEdit(id: string) {
  if (editName.value.trim()) {
    workspaceStore.updateWorkspace(id, { name: editName.value.trim() })
  }
  editingId.value = null
}

function cancelEdit() {
  editingId.value = null
  editName.value = ''
}

async function handleLock() {
  await authStore.lockVault()
  router.push({ name: 'lock' })
}
</script>

<template>
  <div class="min-h-screen bg-background">
    <!-- Header -->
    <header class="border-b border-border bg-card/50 backdrop-blur-sm sticky top-0 z-10">
      <div class="max-w-4xl mx-auto px-6 py-4 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <LayoutGrid class="w-6 h-6 text-primary" />
          <h1 class="text-xl font-bold">{{ t('workspace.selectTitle') }}</h1>
        </div>
        <div class="flex items-center gap-2">
          <button
            @click="router.push('/w/' + workspaceStore.currentWorkspace?.id + '/settings')"
            class="p-2 rounded-lg hover:bg-muted transition-colors text-muted-foreground hover:text-foreground"
            :title="t('nav.settings')"
          >
            <Settings class="w-5 h-5" />
          </button>
          <button
            @click="handleLock"
            class="p-2 rounded-lg hover:bg-muted transition-colors text-muted-foreground hover:text-foreground"
            :title="t('nav.lock')"
          >
            <Lock class="w-5 h-5" />
          </button>
        </div>
      </div>
    </header>

    <!-- Content -->
    <main class="max-w-4xl mx-auto px-6 py-8">
      <!-- Workspace Grid -->
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 mb-8">
        <!-- Existing workspaces -->
        <div
          v-for="ws in workspaceStore.workspaces"
          :key="ws.id"
          @click="selectWorkspace(ws.id)"
          class="group relative bg-card rounded-xl border border-border p-5 cursor-pointer
                 hover:border-primary/50 hover:shadow-md transition-all"
          :style="{ borderLeftColor: ws.color || '#6366f1', borderLeftWidth: '3px' }"
        >
          <!-- Edit mode -->
          <div v-if="editingId === ws.id" class="space-y-3">
            <input
              v-model="editName"
              @click.stop
              @keyup.enter="saveEdit(ws.id)"
              @keyup.escape="cancelEdit"
              class="w-full px-3 py-2 rounded-lg border border-input bg-background text-sm
                     focus:outline-none focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
              autofocus
            />
            <div class="flex gap-2">
              <button @click.stop="saveEdit(ws.id)" class="flex-1 py-1.5 rounded-lg bg-primary text-primary-foreground text-sm hover:bg-primary/90">
                <Check class="w-4 h-4 mx-auto" />
              </button>
              <button @click.stop="cancelEdit" class="flex-1 py-1.5 rounded-lg bg-muted text-sm hover:bg-muted/80">
                <X class="w-4 h-4 mx-auto" />
              </button>
            </div>
          </div>

          <!-- Display mode -->
          <div v-else>
            <div class="flex items-start justify-between mb-3">
              <div class="flex items-center gap-3">
                <span class="text-2xl">{{ ws.icon || '📁' }}</span>
                <div>
                  <h3 class="font-semibold text-sm">{{ ws.name }}</h3>
                  <p class="text-xs text-muted-foreground">
                    {{ new Date(ws.created_at).toLocaleDateString() }}
                  </p>
                </div>
              </div>
            </div>

            <!-- Hover actions -->
            <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                @click="startEdit(ws.id, ws.name, $event)"
                class="p-1.5 rounded-md hover:bg-muted transition-colors"
                :title="t('workspace.rename')"
              >
                <Edit3 class="w-3.5 h-3.5 text-muted-foreground" />
              </button>
              <button
                @click="deleteWorkspace(ws.id, $event)"
                class="p-1.5 rounded-md hover:bg-destructive/10 hover:text-destructive transition-colors"
                :title="t('common.delete')"
              >
                <Trash2 class="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        </div>

        <!-- Add workspace card -->
        <div
          v-if="!showCreateForm"
          @click="showCreateForm = true"
          class="bg-card rounded-xl border-2 border-dashed border-border p-5 cursor-pointer
                 hover:border-primary/50 hover:bg-muted/30 transition-all flex items-center justify-center min-h-[100px]"
        >
          <div class="flex flex-col items-center gap-2 text-muted-foreground">
            <Plus class="w-6 h-6" />
            <span class="text-sm font-medium">{{ t('workspace.create') }}</span>
          </div>
        </div>
      </div>

      <!-- Create Form -->
      <div v-if="showCreateForm" class="bg-card rounded-xl border border-border p-6">
        <h3 class="font-semibold mb-4">{{ t('workspace.createTitle') }}</h3>

        <div class="space-y-4">
          <div>
            <label class="text-sm font-medium mb-1.5 block">{{ t('common.name') }}</label>
            <input
              v-model="newWorkspaceName"
              :placeholder="t('workspace.namePlaceholder')"
              class="w-full rounded-lg border border-input bg-background px-4 py-3 text-sm
                     placeholder:text-muted-foreground
                     focus:outline-none focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
              autofocus
            />
          </div>

          <div>
            <label class="text-sm font-medium mb-1.5 block">{{ t('common.icon') }}</label>
            <div class="flex flex-wrap gap-2">
              <button
                v-for="icon in iconOptions"
                :key="icon"
                @click="newWorkspaceIcon = icon"
                class="w-9 h-9 rounded-lg text-lg flex items-center justify-center border-2 transition-all"
                :class="newWorkspaceIcon === icon ? 'border-primary bg-primary/10' : 'border-border hover:border-muted'"
              >
                {{ icon }}
              </button>
            </div>
          </div>

          <div>
            <label class="text-sm font-medium mb-1.5 block">{{ t('common.color') }}</label>
            <div class="flex gap-2">
              <button
                v-for="color in colorOptions"
                :key="color"
                @click="newWorkspaceColor = color"
                class="w-8 h-8 rounded-full border-2 transition-all"
                :class="newWorkspaceColor === color ? 'border-foreground scale-110' : 'border-transparent'"
                :style="{ backgroundColor: color }"
              />
            </div>
          </div>

          <div class="flex gap-2 pt-2">
            <button
              @click="createWorkspace"
              :disabled="!newWorkspaceName.trim()"
              class="flex-1 rounded-lg bg-primary text-primary-foreground px-4 py-2.5 text-sm font-medium
                     hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {{ t('common.create') }}
            </button>
            <button
              @click="showCreateForm = false"
              class="px-4 py-2.5 text-sm text-muted-foreground hover:text-foreground transition-colors"
            >
              {{ t('common.cancel') }}
            </button>
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-if="workspaceStore.workspaces.length === 0 && !showCreateForm"
           class="text-center py-16 text-muted-foreground">
        <FolderOpen class="w-12 h-12 mx-auto mb-4 opacity-50" />
        <p class="text-lg font-medium">{{ t('workspace.empty') }}</p>
        <p class="text-sm mt-1">{{ t('workspace.emptyPrompt') }}</p>
      </div>
    </main>

    <!-- Delete confirmation modal -->
    <div v-if="pendingDeleteId"
         class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
         @click.self="pendingDeleteId = null">
      <div class="w-full max-w-sm rounded-xl border border-border bg-card p-6 shadow-xl">
        <h2 class="text-lg font-semibold mb-1">{{ t('workspace.deleteTitle') }}</h2>
        <p class="text-sm text-muted-foreground mb-1">
          {{ t('workspace.deleteConfirm', { name: pendingDeleteWorkspace?.name }) }}
        </p>
        <p class="text-sm text-destructive mb-5">{{ t('workspace.deleteWarning') }}</p>
        <div class="flex gap-2">
          <button @click="confirmDelete"
                  class="flex-1 h-9 rounded-md bg-destructive text-destructive-foreground text-sm font-medium
                         hover:bg-destructive/90 transition-colors">
            {{ t('common.confirmDelete') }}
          </button>
          <button @click="pendingDeleteId = null"
                  class="flex-1 h-9 rounded-md border border-input bg-background text-sm
                         hover:bg-accent transition-colors">
            {{ t('common.cancel') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
