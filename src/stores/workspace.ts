import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Workspace {
  id: string
  name: string
  icon: string | null
  color: string | null
  db_file: string
  sort_order: number
  created_at: string
  updated_at: string
}

export const useWorkspaceStore = defineStore('workspace', () => {
  const workspaces = ref<Workspace[]>([])
  const currentWorkspace = ref<Workspace | null>(null)
  const loading = ref(false)

  async function fetchWorkspaces() {
    loading.value = true
    try {
      workspaces.value = await invoke<Workspace[]>('list_workspaces')
    } finally {
      loading.value = false
    }
  }

  async function createWorkspace(name: string, icon?: string, color?: string) {
    const workspace = await invoke<Workspace>('create_workspace', {
      payload: { name, icon, color }
    })
    workspaces.value.push(workspace)
    return workspace
  }

  async function updateWorkspace(id: string, updates: { name?: string; icon?: string; color?: string }) {
    const updated = await invoke<Workspace>('update_workspace', {
      payload: { id, ...updates }
    })
    const idx = workspaces.value.findIndex(w => w.id === id)
    if (idx !== -1) workspaces.value[idx] = updated
    if (currentWorkspace.value?.id === id) currentWorkspace.value = updated
    return updated
  }

  async function deleteWorkspace(id: string) {
    await invoke('delete_workspace', { id })
    workspaces.value = workspaces.value.filter(w => w.id !== id)
    if (currentWorkspace.value?.id === id) currentWorkspace.value = null
  }

  async function switchWorkspace(id: string) {
    const workspace = await invoke<Workspace>('switch_workspace', { id })
    currentWorkspace.value = workspace
    return workspace
  }

  function clearCurrentWorkspace() {
    currentWorkspace.value = null
  }

  return {
    workspaces,
    currentWorkspace,
    loading,
    fetchWorkspaces,
    createWorkspace,
    updateWorkspace,
    deleteWorkspace,
    switchWorkspace,
    clearCurrentWorkspace,
  }
})
