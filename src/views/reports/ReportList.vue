<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import type { Report } from '../../types'
import { Plus, Calendar, FileText } from 'lucide-vue-next'

const router = useRouter()
const { t } = useI18n()
const reports = ref<Report[]>([])
const loading = ref(true)
const activeTab = ref<'daily' | 'weekly'>('daily')
const showCreateModal = ref(false)

// ── date helpers ──────────────────────────────────────────────
function toDateStr(d: Date): string {
  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  return `${y}-${m}-${day}`
}

function getISOWeek(d: Date): number {
  const tmp = new Date(Date.UTC(d.getFullYear(), d.getMonth(), d.getDate()))
  const dow = tmp.getUTCDay() || 7
  tmp.setUTCDate(tmp.getUTCDate() + 4 - dow)
  const jan1 = new Date(Date.UTC(tmp.getUTCFullYear(), 0, 1))
  return Math.ceil(((tmp.getTime() - jan1.getTime()) / 86_400_000 + 1) / 7)
}

function getWeekDay(d: Date, offset: number): Date {
  const dow = d.getDay() || 7          // Sun→7
  const monday = new Date(d)
  monday.setDate(d.getDate() - dow + 1)
  const result = new Date(monday)
  result.setDate(monday.getDate() + offset)
  return result
}

function dailyTitle(d: Date): string {
  const y = d.getFullYear()
  const m = d.getMonth() + 1
  const day = d.getDate()
  return `${y}/${m}/${day} ${t('reports.daily')}`
}

function weeklyTitle(d: Date): string {
  const monday = getWeekDay(d, 0)
  const year = monday.getFullYear()
  const week = getISOWeek(monday)
  return t('reports.weeklyTitleFmt', { year, week })
}

// ── form state ────────────────────────────────────────────────
const today = new Date()

const dailyForm = ref({
  title: dailyTitle(today),
  date: toDateStr(today),
})

const weeklyForm = ref({
  title: weeklyTitle(today),
  week_start: toDateStr(getWeekDay(today, 0)),   // Monday
  week_end: toDateStr(getWeekDay(today, 4)),      // Friday
})

// reset form defaults whenever the modal opens
watch(showCreateModal, (open) => {
  if (!open) return
  const now = new Date()
  if (activeTab.value === 'daily') {
    dailyForm.value = { title: dailyTitle(now), date: toDateStr(now) }
  } else {
    weeklyForm.value = {
      title: weeklyTitle(now),
      week_start: toDateStr(getWeekDay(now, 0)),
      week_end: toDateStr(getWeekDay(now, 4)),
    }
  }
})

// ── data ──────────────────────────────────────────────────────
const filteredReports = computed(() =>
  reports.value.filter(r => r.type === activeTab.value)
)

async function loadData() {
  loading.value = true
  try {
    reports.value = await invoke<Report[]>('list_reports', { type: '' })
  } catch (e) {
    console.error('Failed to load reports:', e)
  } finally {
    loading.value = false
  }
}

async function createReport() {
  try {
    let payload: Record<string, unknown>
    if (activeTab.value === 'daily') {
      if (!dailyForm.value.title) return
      payload = {
        title: dailyForm.value.title,
        type: 'daily',
        content: '',
        date: dailyForm.value.date,
      }
    } else {
      if (!weeklyForm.value.title) return
      payload = {
        title: weeklyForm.value.title,
        type: 'weekly',
        content: '',
        date: weeklyForm.value.week_start,
        week_start: weeklyForm.value.week_start,
        week_end: weeklyForm.value.week_end,
      }
    }
    const created = await invoke<Report>('create_report', { data: payload })
    showCreateModal.value = false
    router.push({ name: 'report-detail', params: { id: created.id } })
  } catch (e) {
    console.error('Failed to create report:', e)
  }
}

function formatDate(r: Report): string {
  if (r.type === 'weekly' && r.week_start && r.week_end) {
    return `${r.week_start} ~ ${r.week_end}`
  }
  return r.date
}

loadData()
</script>

<template>
  <div class="max-w-5xl mx-auto">
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold">{{ t('reports.title') }}</h1>
      <button
        @click="showCreateModal = true"
        class="flex items-center gap-1.5 px-3 py-2 rounded-lg bg-primary text-primary-foreground text-sm hover:bg-primary/90 transition-colors"
      >
        <Plus class="w-4 h-4" />
        {{ activeTab === 'daily' ? t('reports.newDaily') : t('reports.newWeekly') }}
      </button>
    </div>

    <!-- Tabs: only daily / weekly -->
    <div class="flex gap-1 p-1 bg-muted rounded-lg w-fit mb-6">
      <button
        v-for="{ key, labelKey } in [{ key: 'daily', labelKey: 'reports.daily' }, { key: 'weekly', labelKey: 'reports.weekly' }]"
        :key="key"
        @click="activeTab = key as 'daily' | 'weekly'; loadData()"
        class="px-4 py-2 rounded-md text-sm font-medium transition-colors"
        :class="activeTab === key ? 'bg-background shadow-sm' : 'hover:bg-muted/50'"
      >{{ t(labelKey) }}</button>
    </div>

    <!-- Loading -->
    <div v-if="loading" class="space-y-3">
      <div v-for="i in 5" :key="i" class="h-20 bg-muted/50 rounded-lg animate-pulse" />
    </div>

    <!-- Empty -->
    <div v-else-if="filteredReports.length === 0" class="text-center py-16 text-muted-foreground">
      <FileText class="w-12 h-12 mx-auto mb-4 opacity-50" />
      <p class="text-lg font-medium">{{ activeTab === 'daily' ? t('reports.emptyDaily') : t('reports.emptyWeekly') }}</p>
      <p class="text-sm mt-1">{{ t('reports.emptyPrompt') }}</p>
    </div>

    <!-- List -->
    <div v-else class="space-y-2">
      <div
        v-for="r in filteredReports" :key="r.id"
        @click="router.push({ name: 'report-detail', params: { id: r.id } })"
        class="bg-card rounded-xl border border-border p-4 cursor-pointer hover:border-primary/50 transition-all"
      >
        <div class="flex items-center gap-3">
          <Calendar class="w-5 h-5 text-muted-foreground shrink-0" />
          <div class="min-w-0">
            <h3 class="font-medium truncate">{{ r.title }}</h3>
            <p class="text-xs text-muted-foreground">{{ formatDate(r) }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- Create Modal -->
    <div
      v-if="showCreateModal"
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
      @click.self="showCreateModal = false"
    >
      <div class="bg-card rounded-xl border border-border p-6 w-full max-w-sm shadow-xl">
        <h2 class="text-lg font-semibold mb-4">{{ activeTab === 'daily' ? t('reports.newDaily') : t('reports.newWeekly') }}</h2>

        <!-- Daily form -->
        <div v-if="activeTab === 'daily'" class="space-y-3">
          <div>
            <label class="text-xs text-muted-foreground mb-1 block">{{ t('common.title') }}</label>
            <input v-model="dailyForm.title" autofocus
                   class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm focus:outline-none focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
          </div>
          <div>
            <label class="text-xs text-muted-foreground mb-1 block">{{ t('reports.dateLabel') }}</label>
            <input v-model="dailyForm.date" type="date"
                   class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm focus:outline-none focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
          </div>
          <button @click="createReport" :disabled="!dailyForm.title"
                  class="w-full py-2.5 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50 transition-colors">
            {{ t('common.create') }}
          </button>
        </div>

        <!-- Weekly form -->
        <div v-else class="space-y-3">
          <div>
            <label class="text-xs text-muted-foreground mb-1 block">{{ t('common.title') }}</label>
            <input v-model="weeklyForm.title" autofocus
                   class="w-full px-4 py-2.5 rounded-lg border border-input bg-background text-sm focus:outline-none focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
          </div>
          <div class="flex gap-2">
            <div class="flex-1">
              <label class="text-xs text-muted-foreground mb-1 block">{{ t('reports.startDateLabel') }}</label>
              <input v-model="weeklyForm.week_start" type="date"
                     class="w-full px-3 py-2.5 rounded-lg border border-input bg-background text-sm focus:outline-none focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
            </div>
            <div class="flex-1">
              <label class="text-xs text-muted-foreground mb-1 block">{{ t('reports.endDateLabel') }}</label>
              <input v-model="weeklyForm.week_end" type="date"
                     class="w-full px-3 py-2.5 rounded-lg border border-input bg-background text-sm focus:outline-none focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring" />
            </div>
          </div>
          <button @click="createReport" :disabled="!weeklyForm.title"
                  class="w-full py-2.5 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50 transition-colors">
            {{ t('common.create') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
