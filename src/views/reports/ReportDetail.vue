<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import type { Report, Tag } from '../../types'
import { ArrowLeft, Save, Trash2 } from 'lucide-vue-next'
import MarkdownEditor from '../../components/MarkdownEditor.vue'
import TagInput from '../../components/TagInput.vue'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()

const report = ref<Report | null>(null)
const loading = ref(true)
const saving = ref(false)
const showDeleteConfirm = ref(false)
const reportTags = ref<Tag[]>([])

const editForm = ref({ title: '', content: '' })

const hasChanges = computed(() =>
  report.value && (
    editForm.value.title !== report.value.title ||
    editForm.value.content !== report.value.content
  )
)

const typeLabel = computed(() => {
  const type = report.value?.type
  return type === 'daily' ? t('reports.daily') : type === 'weekly' ? t('reports.weekly') : t('reports.detailTitle')
})

const dateMeta = computed(() => {
  if (!report.value) return ''
  if (report.value.type === 'weekly' && report.value.week_start && report.value.week_end) {
    return `${report.value.week_start} ~ ${report.value.week_end}`
  }
  return report.value.date
})

function formatDate(iso: string) {
  const d = new Date(iso)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth()+1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}

async function loadReport() {
  loading.value = true
  try {
    report.value = await invoke<Report>('get_report', { id: route.params.id })
    editForm.value = { title: report.value.title, content: report.value.content }
    reportTags.value = await invoke<Tag[]>('get_entity_tags', {
      entityType: 'report', entityId: route.params.id as string,
    })
  } catch (e) {
    console.error('Failed to load report:', e)
  } finally {
    loading.value = false
  }
}

async function saveReport() {
  if (!report.value || !hasChanges.value) return
  saving.value = true
  try {
    await invoke('update_report', {
      id: report.value.id,
      data: { title: editForm.value.title, content: editForm.value.content },
    })
    report.value = { ...report.value, ...editForm.value }
  } catch (e) {
    console.error('Failed to save report:', e)
  } finally {
    saving.value = false
  }
}

async function deleteReport() {
  if (!report.value) return
  try {
    await invoke('delete_report', { id: report.value.id })
    router.back()
  } catch (e) {
    console.error('Failed to delete report:', e)
  }
}

onMounted(loadReport)
</script>

<template>
  <div class="max-w-3xl mx-auto">

    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div class="flex items-center gap-3">
        <button @click="router.back()"
                class="inline-flex items-center justify-center rounded-md h-9 w-9
                       text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors">
          <ArrowLeft class="w-4 h-4" />
        </button>
        <h1 class="text-xl font-semibold tracking-tight">
          {{ report?.title || t('reports.detailTitle') }}
        </h1>
        <span v-if="report"
              class="px-2 py-0.5 rounded-md text-xs font-medium bg-muted text-muted-foreground">
          {{ typeLabel }}
        </span>
      </div>
      <div class="flex items-center gap-1.5">
        <button @click="showDeleteConfirm = true"
                class="inline-flex items-center justify-center rounded-md h-9 w-9
                       text-muted-foreground hover:bg-destructive/10 hover:text-destructive transition-colors">
          <Trash2 class="w-4 h-4" />
        </button>
        <button @click="saveReport" :disabled="!hasChanges || saving"
                class="inline-flex items-center gap-1.5 rounded-md bg-primary px-3 h-9 text-sm
                       font-medium text-primary-foreground shadow hover:bg-primary/90
                       disabled:pointer-events-none disabled:opacity-50 transition-colors">
          <Save class="w-4 h-4" />
          {{ saving ? t('common.saving') : t('common.save') }}
        </button>
      </div>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="rounded-xl border border-border bg-card p-6 space-y-4">
      <div v-for="i in 5" :key="i" class="h-10 rounded-md bg-muted animate-pulse" />
    </div>

    <!-- Form card -->
    <div v-else-if="report" class="rounded-xl border border-border bg-card shadow-sm">
      <div class="p-6 space-y-5">

        <!-- Title -->
        <div class="space-y-2">
          <label class="text-sm font-medium leading-none">{{ t('common.title') }}</label>
          <input v-model="editForm.title" :placeholder="t('reports.detailTitle')"
                 class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm
                        placeholder:text-muted-foreground focus-visible:outline-none
                        focus-visible:ring-1 focus-visible:ring-ring transition-colors" />
        </div>

        <!-- Date meta (read-only) -->
        <div class="space-y-2">
          <label class="text-sm font-medium leading-none">
            {{ report.type === 'weekly' ? t('reports.period') : t('reports.dateLabel') }}
          </label>
          <div class="flex h-10 w-full items-center rounded-md border border-input bg-muted/40 px-3 text-sm text-muted-foreground select-none">
            {{ dateMeta }}
          </div>
        </div>

        <!-- Tags -->
        <div class="space-y-2">
          <label class="text-sm font-medium leading-none">{{ t('common.tags') }}</label>
          <TagInput v-model="reportTags" entity-type="report" :entity-id="report.id" />
        </div>

        <!-- Content -->
        <div class="space-y-2">
          <label class="text-sm font-medium leading-none">{{ t('common.content') }}</label>
          <div class="h-[480px] rounded-md border border-input overflow-hidden">
            <MarkdownEditor v-model="editForm.content"
                            :placeholder="t('reports.contentPlaceholder')"
                            variant="flat"
                            class="h-full" />
          </div>
        </div>

      </div>

      <!-- Footer meta -->
      <div class="border-t border-border px-6 py-3">
        <p class="text-xs text-muted-foreground">
          {{ t('common.createdAt') }}{{ formatDate(report.created_at) }}
          · {{ t('common.updatedAt') }}{{ formatDate(report.updated_at) }}
        </p>
      </div>
    </div>

    <!-- Delete confirmation modal -->
    <div v-if="showDeleteConfirm"
         class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
         @click.self="showDeleteConfirm = false">
      <div class="w-full max-w-sm rounded-xl border border-border bg-card p-6 shadow-xl">
        <h2 class="text-lg font-semibold mb-1">
          {{ report?.type === 'daily' ? t('reports.deleteDaily') : t('reports.deleteWeekly') }}
        </h2>
        <p class="text-sm text-muted-foreground mb-1">
          {{ t('reports.deleteConfirm', { name: report?.title }) }}
        </p>
        <p class="text-sm text-destructive mb-5">{{ t('common.irreversible') }}</p>
        <div class="flex gap-2">
          <button @click="deleteReport"
                  class="flex-1 h-9 rounded-md bg-destructive text-destructive-foreground text-sm font-medium
                         hover:bg-destructive/90 transition-colors">
            {{ t('common.confirmDelete') }}
          </button>
          <button @click="showDeleteConfirm = false"
                  class="flex-1 h-9 rounded-md border border-input bg-background text-sm
                         hover:bg-accent transition-colors">
            {{ t('common.cancel') }}
          </button>
        </div>
      </div>
    </div>

  </div>
</template>
