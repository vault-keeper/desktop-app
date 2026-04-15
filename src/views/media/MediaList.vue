<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import type { MediaAsset } from '../../types'
import {
  Upload, Image as ImageIcon, Music, Video, Trash2,
  Grid, List, X, Play, FolderOpen
} from 'lucide-vue-next'

const { t } = useI18n()

const assets = ref<MediaAsset[]>([])
const loading = ref(true)
const viewMode = ref<'grid' | 'list'>('grid')
const selectedAsset = ref<MediaAsset | null>(null)
const pendingDeleteId = ref<string | null>(null)

// ── media type helpers ────────────────────────────────────────
function isImage(asset: MediaAsset) { return asset.mime_type.startsWith('image/') }
function isAudio(asset: MediaAsset) { return asset.mime_type.startsWith('audio/') }
function isVideo(asset: MediaAsset) { return asset.mime_type.startsWith('video/') }

function mimeLabel(asset: MediaAsset): string {
  if (isImage(asset)) return t('media.imageType')
  if (isAudio(asset)) return t('media.audioType')
  if (isVideo(asset)) return t('media.videoType')
  return asset.mime_type
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1048576) return (bytes / 1024).toFixed(1) + ' KB'
  if (bytes < 1073741824) return (bytes / 1048576).toFixed(1) + ' MB'
  return (bytes / 1073741824).toFixed(1) + ' GB'
}

// ── preview URL ───────────────────────────────────────────────
// Returns a URL the WebView can display:
// - local: asset:// URL via convertFileSrc (asset protocol enabled in tauri.conf.json)
// - s3: direct storage_path URL
function previewSrc(asset: MediaAsset): string | null {
  if (asset.storage_type === 'local') {
    if (!asset.absolute_path) return null
    return convertFileSrc(asset.absolute_path)
  }
  // S3: storage_path is a public/signed URL
  return asset.storage_path || null
}

// ── data ──────────────────────────────────────────────────────
async function loadData() {
  loading.value = true
  try {
    assets.value = await invoke<MediaAsset[]>('list_media_assets')
  } catch (e) {
    console.error('Failed to load media:', e)
  } finally {
    loading.value = false
  }
}

// ── upload ────────────────────────────────────────────────────
async function uploadFile() {
  try {
    const selected = await openDialog({
      multiple: true,
      filters: [{
        name: `${t('media.imageType')} / ${t('media.audioType')} / ${t('media.videoType')}`,
        extensions: [
          'png', 'jpg', 'jpeg', 'gif', 'webp', 'svg',
          'mp3', 'wav', 'ogg', 'flac', 'm4a', 'aac',
          'mp4', 'webm', 'mov', 'mkv',
        ]
      }]
    })
    if (!selected) return
    const paths = Array.isArray(selected) ? selected : [selected]
    for (const path of paths) {
      await invoke('upload_media', {
        payload: { file_path: path, description: '' }
      })
    }
    await loadData()
  } catch (e) {
    console.error('Failed to upload:', e)
  }
}

// ── delete ────────────────────────────────────────────────────
async function deleteAsset(id: string) {
  pendingDeleteId.value = id
}

async function revealAsset(asset: MediaAsset) {
  if (!asset.absolute_path) return
  await revealItemInDir(asset.absolute_path)
}

async function confirmDelete() {
  if (!pendingDeleteId.value) return
  const id = pendingDeleteId.value
  pendingDeleteId.value = null
  await invoke('delete_media_asset', { id })
  if (selectedAsset.value?.id === id) selectedAsset.value = null
  await loadData()
}

onMounted(loadData)
</script>

<template>
  <div class="max-w-5xl mx-auto">

    <!-- Header -->
    <div class="flex items-center justify-between mb-4">
      <h1 class="text-2xl font-bold">{{ t('media.title') }}</h1>
      <div class="flex items-center gap-2">
        <button @click="viewMode = 'grid'"
                class="p-2 rounded-lg transition-colors"
                :class="viewMode === 'grid'
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted hover:bg-muted/80'">
          <Grid class="w-4 h-4" />
        </button>
        <button @click="viewMode = 'list'"
                class="p-2 rounded-lg transition-colors"
                :class="viewMode === 'list'
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted hover:bg-muted/80'">
          <List class="w-4 h-4" />
        </button>
        <button @click="uploadFile"
                class="flex items-center gap-1.5 px-3 py-2 rounded-lg bg-primary
                       text-primary-foreground text-sm hover:bg-primary/90 transition-colors">
          <Upload class="w-4 h-4" />
          {{ t('media.uploadBtn') }}
        </button>
      </div>
    </div>

    <!-- Loading skeleton -->
    <div v-if="loading" class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-4">
      <div v-for="i in 8" :key="i" class="aspect-square bg-muted/50 rounded-xl animate-pulse" />
    </div>

    <!-- Empty state -->
    <div v-else-if="assets.length === 0"
         class="text-center py-16 text-muted-foreground">
      <ImageIcon class="w-12 h-12 mx-auto mb-4 opacity-40" />
      <p class="text-lg font-medium">{{ t('media.empty') }}</p>
      <p class="text-sm mt-1">{{ t('media.fileTypesInfo') }}</p>
      <button @click="uploadFile"
              class="mt-4 inline-flex items-center gap-1.5 px-4 py-2 rounded-lg bg-primary
                     text-primary-foreground text-sm hover:bg-primary/90 transition-colors">
        <Upload class="w-4 h-4" />
        {{ t('media.uploadPrompt') }}
      </button>
    </div>

    <!-- Grid view -->
    <div v-else-if="viewMode === 'grid'"
         class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-4">
      <div v-for="asset in assets" :key="asset.id"
           @click="selectedAsset = asset"
           class="group bg-card rounded-xl border border-border overflow-hidden
                  cursor-pointer hover:border-primary/50 hover:shadow-sm transition-all">
        <!-- Thumbnail -->
        <div class="aspect-square bg-muted flex items-center justify-center
                    relative overflow-hidden">
          <!-- Image thumbnail -->
          <img v-if="isImage(asset) && previewSrc(asset)"
               :src="previewSrc(asset)!"
               :alt="asset.filename"
               class="w-full h-full object-cover" />
          <!-- Video thumbnail: show first-frame via video element -->
          <video v-else-if="isVideo(asset) && previewSrc(asset)"
                 :src="previewSrc(asset)!"
                 class="w-full h-full object-cover"
                 preload="metadata"
                 muted />
          <!-- Audio icon -->
          <div v-else-if="isAudio(asset)"
               class="flex flex-col items-center gap-2 text-muted-foreground">
            <Music class="w-10 h-10 opacity-60" />
          </div>
          <!-- Fallback icon -->
          <div v-else
               class="flex flex-col items-center gap-2 text-muted-foreground">
            <ImageIcon class="w-8 h-8 opacity-30" />
          </div>
          <!-- Hover overlay -->
          <div class="absolute inset-0 bg-black/0 group-hover:bg-black/25
                      transition-colors flex items-center justify-center gap-2">
            <Play class="w-8 h-8 text-white opacity-0 group-hover:opacity-90
                         transition-opacity drop-shadow-lg" />
            <button v-if="asset.absolute_path"
                    @click.stop="revealAsset(asset)"
                    :title="t('media.showInFolder')"
                    class="absolute bottom-2 right-2 p-1.5 rounded-md bg-black/40 text-white
                           opacity-0 group-hover:opacity-100 hover:bg-black/60 transition-all">
              <FolderOpen class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
        <!-- Info row -->
        <div class="px-2.5 py-2">
          <p class="text-xs font-medium truncate">{{ asset.filename }}</p>
          <div class="flex items-center justify-between mt-0.5">
            <span class="text-xs text-muted-foreground">{{ formatSize(asset.size) }}</span>
            <span class="text-xs text-muted-foreground/70 bg-muted px-1.5 py-0.5 rounded">
              {{ mimeLabel(asset) }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- List view -->
    <div v-else class="space-y-2">
      <div v-for="asset in assets" :key="asset.id"
           @click="selectedAsset = asset"
           class="group flex items-center gap-3 bg-card rounded-xl border border-border p-3
                  cursor-pointer hover:border-primary/50 transition-all">
        <!-- Thumb -->
        <div class="w-12 h-12 rounded-lg bg-muted flex items-center
                    justify-center shrink-0 overflow-hidden">
          <img v-if="isImage(asset) && previewSrc(asset)"
               :src="previewSrc(asset)!"
               :alt="asset.filename"
               class="w-full h-full object-cover" />
          <video v-else-if="isVideo(asset) && previewSrc(asset)"
                 :src="previewSrc(asset)!"
                 class="w-full h-full object-cover"
                 preload="metadata" muted />
          <Music v-else-if="isAudio(asset)" class="w-6 h-6 text-muted-foreground" />
          <Video v-else-if="isVideo(asset)" class="w-6 h-6 text-muted-foreground" />
          <ImageIcon v-else class="w-6 h-6 text-muted-foreground opacity-40" />
        </div>
        <!-- Info -->
        <div class="flex-1 min-w-0">
          <p class="text-sm font-medium truncate">{{ asset.filename }}</p>
          <p class="text-xs text-muted-foreground">
            {{ formatSize(asset.size) }} · {{ mimeLabel(asset) }}
          </p>
        </div>
        <!-- Reveal + Delete -->
        <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-all shrink-0">
          <button v-if="asset.absolute_path"
                  @click.stop="revealAsset(asset)"
                  :title="t('media.showInFolder')"
                  class="p-1.5 rounded-md text-muted-foreground hover:bg-muted hover:text-foreground transition-colors">
            <FolderOpen class="w-4 h-4" />
          </button>
          <button @click.stop="deleteAsset(asset.id)"
                  class="p-1.5 rounded-md text-muted-foreground hover:bg-destructive/10 hover:text-destructive transition-colors">
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>

    <!-- ── Lightbox / Detail modal ── -->
    <div v-if="selectedAsset"
         class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 p-4"
         @click.self="selectedAsset = null">
      <div class="bg-card rounded-xl border border-border shadow-2xl
                  w-full max-w-3xl overflow-hidden">

        <!-- Preview -->
        <div class="relative bg-black/5 dark:bg-black/30 flex items-center
                    justify-center min-h-56 p-6">

          <!-- Image -->
          <img v-if="isImage(selectedAsset) && previewSrc(selectedAsset)"
               :src="previewSrc(selectedAsset)!"
               :alt="selectedAsset.filename"
               class="max-w-full max-h-[65vh] rounded-lg object-contain shadow" />

          <!-- Video player -->
          <video v-else-if="isVideo(selectedAsset) && previewSrc(selectedAsset)"
                 :src="previewSrc(selectedAsset)!"
                 controls
                 class="max-w-full max-h-[65vh] rounded-lg shadow" />

          <!-- Audio player -->
          <div v-else-if="isAudio(selectedAsset)"
               class="w-full flex flex-col items-center gap-5 py-8">
            <div class="w-24 h-24 rounded-full bg-primary/10
                        flex items-center justify-center">
              <Music class="w-12 h-12 text-primary" />
            </div>
            <p class="text-sm font-medium text-center">{{ selectedAsset.filename }}</p>
            <audio v-if="previewSrc(selectedAsset)"
                   :src="previewSrc(selectedAsset)!"
                   controls
                   class="w-full max-w-md" />
            <p v-else class="text-sm text-muted-foreground">{{ t('media.audioError') }}</p>
          </div>

          <!-- No preview -->
          <div v-else class="flex flex-col items-center gap-3 py-10 text-muted-foreground">
            <ImageIcon class="w-14 h-14 opacity-25" />
            <p class="text-sm">{{ t('media.previewError') }}</p>
          </div>

          <!-- Close button -->
          <button @click="selectedAsset = null"
                  class="absolute top-3 right-3 p-1.5 rounded-md bg-black/30 text-white
                         hover:bg-black/50 transition-colors">
            <X class="w-4 h-4" />
          </button>
        </div>

        <!-- Footer -->
        <div class="flex items-center justify-between px-5 py-3 border-t border-border">
          <div class="min-w-0">
            <p class="font-medium text-sm truncate">{{ selectedAsset.filename }}</p>
            <p class="text-xs text-muted-foreground">
              {{ formatSize(selectedAsset.size) }} · {{ mimeLabel(selectedAsset) }}
            </p>
          </div>
          <div class="flex items-center gap-2 shrink-0">
            <button v-if="selectedAsset.absolute_path"
                    @click="revealAsset(selectedAsset)"
                    class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm
                           text-muted-foreground border border-border
                           hover:bg-muted transition-colors">
              <FolderOpen class="w-4 h-4" />
              {{ t('media.showInFolder') }}
            </button>
            <button @click="deleteAsset(selectedAsset.id)"
                    class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm
                           text-destructive border border-destructive/30
                           hover:bg-destructive/10 transition-colors">
              <Trash2 class="w-4 h-4" />
              {{ t('common.delete') }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Delete file modal -->
    <div v-if="pendingDeleteId"
         class="fixed inset-0 z-[60] flex items-center justify-center bg-black/50"
         @click.self="pendingDeleteId = null">
      <div class="w-full max-w-sm rounded-xl border border-border bg-card p-6 shadow-xl">
        <h2 class="text-lg font-semibold mb-1">{{ t('media.deleteTitle') }}</h2>
        <p class="text-sm text-muted-foreground mb-1">
          {{ t('media.deleteConfirm', { name: assets.find(a => a.id === pendingDeleteId)?.filename }) }}
        </p>
        <p class="text-sm text-destructive mb-5">{{ t('common.irreversible') }}</p>
        <div class="flex gap-2">
          <button @click="confirmDelete"
                  class="flex-1 h-9 rounded-md bg-destructive text-destructive-foreground text-sm font-medium hover:bg-destructive/90 transition-colors">
            {{ t('common.confirmDelete') }}
          </button>
          <button @click="pendingDeleteId = null"
                  class="flex-1 h-9 rounded-md border border-input bg-background text-sm hover:bg-accent transition-colors">
            {{ t('common.cancel') }}
          </button>
        </div>
      </div>
    </div>

  </div>
</template>
