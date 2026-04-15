<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronRight, ChevronDown, FolderOpen, Folder, Plus, MoreHorizontal, Pencil, Trash2 } from 'lucide-vue-next'
import type { AnyGroup } from '../../types'

export interface GroupNode {
  group: AnyGroup
  children: GroupNode[]
}

const props = defineProps<{
  groups: AnyGroup[]
  selectedGroupId: string | null
  showUngrouped?: boolean
}>()

const emit = defineEmits<{
  (e: 'select', groupId: string | null): void
  (e: 'create', parentId: string | null): void
  (e: 'edit', group: AnyGroup): void
  (e: 'delete', group: AnyGroup): void
}>()

const { t } = useI18n()
const expandedIds = ref<Set<string>>(new Set())

const tree = computed(() => {
  const byParent = new Map<string | null, AnyGroup[]>()
  for (const g of props.groups) {
    const pid = g.parent_id
    if (!byParent.has(pid)) byParent.set(pid, [])
    byParent.get(pid)!.push(g)
  }

  function build(parentId: string | null): GroupNode[] {
    const children = byParent.get(parentId) || []
    return children
      .sort((a, b) => a.sort_order - b.sort_order)
      .map((group) => ({
        group,
        children: build(group.id),
      }))
  }

  return build(null)
})

function toggleExpand(id: string) {
  if (expandedIds.value.has(id)) {
    expandedIds.value.delete(id)
  } else {
    expandedIds.value.add(id)
  }
}

function isExpanded(id: string) {
  return expandedIds.value.has(id)
}

function handleSelect(groupId: string | null) {
  emit('select', groupId)
}

const contextMenuGroupId = ref<string | null>(null)

function toggleContextMenu(id: string, e: Event) {
  e.stopPropagation()
  contextMenuGroupId.value = contextMenuGroupId.value === id ? null : id
}

function closeContextMenu() {
  contextMenuGroupId.value = null
}
</script>

<template>
  <div class="space-y-0.5 text-sm" @click="closeContextMenu">
    <!-- Ungrouped option -->
    <button
      v-if="showUngrouped !== false"
      @click="handleSelect(null)"
      class="w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-left transition-colors"
      :class="selectedGroupId === null ? 'bg-accent text-accent-foreground' : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'"
    >
      <Folder class="w-4 h-4 shrink-0" />
      <span>{{ t('groupTree.ungrouped') }}</span>
    </button>

    <!-- Tree nodes -->
    <div v-for="node in tree" :key="node.group.id">
      <GroupTreeNode
        :node="node"
        :depth="0"
        :selected-group-id="selectedGroupId"
        :expanded-ids="expandedIds"
        :context-menu-group-id="contextMenuGroupId"
        @toggle-expand="toggleExpand"
        @select="handleSelect"
        @create="emit('create', $event)"
        @edit="emit('edit', $event); closeContextMenu()"
        @delete="emit('delete', $event); closeContextMenu()"
        @toggle-context-menu="toggleContextMenu"
      />
    </div>

    <!-- Add group button -->
    <button
      @click="emit('create', null)"
      class="w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors"
    >
      <Plus class="w-4 h-4 shrink-0" />
      <span>{{ t('groupTree.newGroup') }}</span>
    </button>
  </div>
</template>

<script lang="ts">
// Named recursive child component
import { defineComponent } from 'vue'

const GroupTreeNode = defineComponent({
  name: 'GroupTreeNode',
  props: {
    node: { type: Object as () => GroupNode, required: true },
    depth: { type: Number, required: true },
    selectedGroupId: { type: String as () => string | null, default: null },
    expandedIds: { type: Object as () => Set<string>, required: true },
    contextMenuGroupId: { type: String as () => string | null, default: null },
  },
  emits: ['toggle-expand', 'select', 'create', 'edit', 'delete', 'toggle-context-menu'],
  setup(props, { emit }) {
    const FolderOpen_ = FolderOpen
    const Folder_ = Folder
    const ChevronRight_ = ChevronRight
    const ChevronDown_ = ChevronDown
    const Pencil_ = Pencil
    const Trash2_ = Trash2
    const Plus_ = Plus
    const MoreHorizontal_ = MoreHorizontal

    return {
      FolderOpen: FolderOpen_,
      Folder: Folder_,
      ChevronRight: ChevronRight_,
      ChevronDown: ChevronDown_,
      Pencil: Pencil_,
      Trash2: Trash2_,
      Plus: Plus_,
      MoreHorizontal: MoreHorizontal_,
    }
  },
  template: `
    <div>
      <div
        class="w-full flex items-center gap-1 px-2 py-1.5 rounded-md text-left transition-colors relative group"
        :class="selectedGroupId === node.group.id ? 'bg-accent text-accent-foreground' : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'"
        :style="{ paddingLeft: (depth * 16 + 8) + 'px' }"
        @click="emit('select', node.group.id)"
      >
        <button
          v-if="node.children.length"
          @click.stop="emit('toggle-expand', node.group.id)"
          class="p-0.5 rounded hover:bg-accent transition-colors"
        >
          <ChevronDown v-if="expandedIds.has(node.group.id)" class="w-3.5 h-3.5" />
          <ChevronRight v-else class="w-3.5 h-3.5" />
        </button>
        <span v-else class="w-5" />

        <FolderOpen v-if="expandedIds.has(node.group.id) && node.children.length" class="w-4 h-4 shrink-0" />
        <Folder v-else class="w-4 h-4 shrink-0" />

        <span class="flex-1 truncate">{{ node.group.name }}</span>

        <div class="opacity-0 group-hover:opacity-100 flex items-center gap-0.5 transition-opacity">
          <button
            @click.stop="emit('create', node.group.id)"
            class="p-1 rounded hover:bg-accent transition-colors"
            :title="$t('groupTree.newSubGroup')"
          >
            <Plus class="w-3 h-3" />
          </button>
          <button
            @click.stop="emit('toggle-context-menu', node.group.id, $event)"
            class="p-1 rounded hover:bg-accent transition-colors"
          >
            <MoreHorizontal class="w-3 h-3" />
          </button>
        </div>

        <!-- Context menu -->
        <div
          v-if="contextMenuGroupId === node.group.id"
          class="absolute right-0 top-full z-50 mt-1 bg-popover border border-border rounded-lg shadow-md py-1 min-w-[120px]"
          @click.stop
        >
          <button
            @click="emit('edit', node.group)"
            class="w-full flex items-center gap-2 px-3 py-1.5 text-sm hover:bg-accent transition-colors"
          >
            <Pencil class="w-3 h-3" />
            {{ $t('common.edit') }}
          </button>
          <button
            @click="emit('delete', node.group)"
            class="w-full flex items-center gap-2 px-3 py-1.5 text-sm text-destructive hover:bg-destructive/10 transition-colors"
          >
            <Trash2 class="w-3 h-3" />
            {{ $t('common.delete') }}
          </button>
        </div>
      </div>

      <div v-if="expandedIds.has(node.group.id) && node.children.length" class="space-y-0.5">
        <GroupTreeNode
          v-for="child in node.children"
          :key="child.group.id"
          :node="child"
          :depth="depth + 1"
          :selected-group-id="selectedGroupId"
          :expanded-ids="expandedIds"
          :context-menu-group-id="contextMenuGroupId"
          @toggle-expand="emit('toggle-expand', $event)"
          @select="emit('select', $event)"
          @create="emit('create', $event)"
          @edit="emit('edit', $event)"
          @delete="emit('delete', $event)"
          @toggle-context-menu="emit('toggle-context-menu', $event)"
        />
      </div>
    </div>
  `,
})

export default { components: { GroupTreeNode } }
</script>
