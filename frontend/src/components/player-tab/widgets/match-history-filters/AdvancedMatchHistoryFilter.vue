<template>
  <div class="space-y-4 text-xs text-slate-200 select-none">
    <!-- Presets bar -->
    <div class="flex items-center justify-between pb-2 border-b border-slate-800">
      <span class="text-xs font-bold text-slate-300">快速组合预设</span>
      <div class="flex items-center space-x-1.5">
        <button
          v-for="p in presets"
          :key="p.name"
          @click="applyPreset(p)"
          class="px-2 py-0.8 rounded-lg bg-slate-900 hover:bg-slate-800 text-[11px] text-slate-400 hover:text-cyan-300 border border-slate-800 transition active:scale-95"
        >
          {{ p.name }}
        </button>
      </div>
    </div>

    <!-- Tree View -->
    <div class="space-y-3">
      <CombinatorNodeRenderer
        :node-id="state.rootId"
        :root-id="state.rootId"
        :node-map="state.nodeMap"
        @add-node="handleAddNode"
        @remove="handleRemoveNode"
        @update-params="handleUpdateParams"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { AdvancedFilterState, AstNode } from './filter-state'
import CombinatorNodeRenderer from './combinator-components/CombinatorNodeRenderer.vue'

const props = defineProps<{
  modelValue: AdvancedFilterState
}>()

const emit = defineEmits<{
  'update:modelValue': [val: AdvancedFilterState]
}>()

const state = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
})

let nextNodeId = 1

function createId(prefix = 'node'): string {
  return `${prefix}_${Date.now()}_${nextNodeId++}`
}

function handleAddNode({ parentId, type }: { parentId: string; type: AstNode['type'] }) {
  const newId = createId(type)
  const newNode: AstNode = {
    id: newId,
    type,
    children: type === 'and' || type === 'or' ? [] : undefined,
    params:
      type === 'isChampion'
        ? { championId: 157 }
        : type === 'isPosition'
        ? { position: 'TOP' }
        : type === 'numberBetween'
        ? { metric: 'kda', min: 4, max: 100 }
        : {}
  }

  const updatedMap = { ...state.value.nodeMap }
  updatedMap[newId] = newNode

  const parent = updatedMap[parentId]
  if (parent) {
    updatedMap[parentId] = {
      ...parent,
      children: [...(parent.children || []), newId]
    }
  }

  state.value = {
    ...state.value,
    nodeMap: updatedMap
  }
}

function handleRemoveNode(nodeId: string) {
  if (nodeId === state.value.rootId) return

  const updatedMap = { ...state.value.nodeMap }

  // Remove from parents
  for (const key of Object.keys(updatedMap)) {
    const n = updatedMap[key]
    if (n.children && n.children.includes(nodeId)) {
      updatedMap[key] = {
        ...n,
        children: n.children.filter((id) => id !== nodeId)
      }
    }
  }

  // Remove node itself
  delete updatedMap[nodeId]

  state.value = {
    ...state.value,
    nodeMap: updatedMap
  }
}

function handleUpdateParams({ nodeId, params }: { nodeId: string; params: Record<string, any> }) {
  const node = state.value.nodeMap[nodeId]
  if (!node) return

  state.value = {
    ...state.value,
    nodeMap: {
      ...state.value.nodeMap,
      [nodeId]: {
        ...node,
        params
      }
    }
  }
}

const presets = [
  {
    name: '通天代 carry (KDA > 5 且 胜利)',
    setup: () => {
      const rootId = 'node_root'
      const id1 = createId('isWin')
      const id2 = createId('num')
      return {
        rootId,
        nodeMap: {
          [rootId]: { id: rootId, type: 'and' as const, children: [id1, id2] },
          [id1]: { id: id1, type: 'isWin' as const },
          [id2]: { id: id2, type: 'numberBetween' as const, params: { metric: 'kda', min: 5, max: 999 } }
        }
      }
    }
  },
  {
    name: '高输出对局 (伤害 > 25000)',
    setup: () => {
      const rootId = 'node_root'
      const id1 = createId('num')
      return {
        rootId,
        nodeMap: {
          [rootId]: { id: rootId, type: 'and' as const, children: [id1] },
          [id1]: { id: id1, type: 'numberBetween' as const, params: { metric: 'damage', min: 25000, max: 999999 } }
        }
      }
    }
  }
]

function applyPreset(p: { setup: () => AdvancedFilterState }) {
  state.value = p.setup()
}
</script>
