<template>
  <div v-if="node" class="relative group">
    <!-- AND / OR group -->
    <div
      v-if="node.type === 'and' || node.type === 'or'"
      class="p-2.5 rounded-xl border space-y-2"
      :class="node.type === 'and' ? 'bg-slate-900/80 border-cyan-500/30' : 'bg-slate-900/80 border-amber-500/30'"
    >
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-2">
          <span
            class="px-2 py-0.5 rounded font-mono font-bold text-[10px]"
            :class="node.type === 'and' ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40' : 'bg-amber-500/20 text-amber-300 border border-amber-500/40'"
          >
            {{ node.type.toUpperCase() }} ({{ node.type === 'and' ? '全部满足' : '任一满足' }})
          </span>
          <span class="text-[11px] text-slate-400">
            包含 {{ node.children?.length || 0 }} 个组合条件
          </span>
        </div>

        <div class="flex items-center space-x-1">
          <!-- Add condition dropdown -->
          <n-dropdown
            trigger="click"
            :options="addNodeOptions"
            size="small"
            @select="handleAddChild"
          >
            <button
              class="px-2 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white text-[11px] font-semibold flex items-center space-x-1 transition"
            >
              <Plus class="w-3 h-3" />
              <span>添加子条件</span>
            </button>
          </n-dropdown>

          <!-- Remove this node (if not root) -->
          <button
            v-if="nodeId !== rootId"
            @click="$emit('remove', nodeId)"
            class="p-1 rounded-lg hover:bg-slate-800 text-slate-500 hover:text-rose-400 transition"
            title="移除当前组合组"
          >
            <X class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      <!-- Children list -->
      <div v-if="node.children && node.children.length > 0" class="space-y-2 pl-3 border-l-2 border-slate-800">
        <CombinatorNodeRenderer
          v-for="childId in node.children"
          :key="childId"
          :node-id="childId"
          :root-id="rootId"
          :node-map="nodeMap"
          @add-node="$emit('addNode', $event)"
          @remove="$emit('remove', $event)"
          @update-params="$emit('updateParams', $event)"
        />
      </div>

      <!-- Empty children placeholder -->
      <div v-else class="py-2 text-center text-[11px] text-slate-500 border border-dashed border-slate-800 rounded-lg">
        尚未添加任何过滤条件，点击右上角 “添加子条件”
      </div>
    </div>

    <!-- Leaf: Champion -->
    <div
      v-else-if="node.type === 'isChampion'"
      class="p-2 px-3 rounded-xl bg-slate-950/70 border border-slate-800 flex items-center justify-between text-xs"
    >
      <div class="flex items-center space-x-2.5">
        <span class="text-amber-400 font-bold text-[11px]">英雄限制:</span>
        <ChampionIcon :champion-id="node.params?.championId || 157" :size="24" round />
        <span class="font-bold text-slate-200">{{ getChampionName(node.params?.championId || 157) }}</span>
      </div>
      <div class="flex items-center space-x-2">
        <select
          :value="node.params?.championId || 157"
          @change="updateParams({ championId: Number(($event.target as HTMLSelectElement).value) })"
          class="bg-slate-900 border border-slate-800 rounded px-1.5 py-0.5 text-[11px] text-slate-300 outline-none"
        >
          <option v-for="(name, id) in championEntries" :key="id" :value="Number(id)">
            {{ name }}
          </option>
        </select>
        <button @click="$emit('remove', nodeId)" class="hover:text-rose-400 text-slate-500">
          <X class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Leaf: Win Result -->
    <div
      v-else-if="node.type === 'isWin' || node.type === 'isLoss'"
      class="p-2 px-3 rounded-xl bg-slate-950/70 border border-slate-800 flex items-center justify-between text-xs"
    >
      <div class="flex items-center space-x-2">
        <span class="text-indigo-400 font-bold text-[11px]">胜负判定:</span>
        <span
          class="px-2 py-0.5 rounded text-[10px] font-bold"
          :class="node.type === 'isWin' ? 'bg-emerald-950 border border-emerald-500/50 text-emerald-300' : 'bg-rose-950 border border-rose-500/50 text-rose-300'"
        >
          {{ node.type === 'isWin' ? '必须胜利 (Victory)' : '必须失败 (Defeat)' }}
        </span>
      </div>
      <button @click="$emit('remove', nodeId)" class="hover:text-rose-400 text-slate-500">
        <X class="w-3.5 h-3.5" />
      </button>
    </div>

    <!-- Leaf: Position -->
    <div
      v-else-if="node.type === 'isPosition'"
      class="p-2 px-3 rounded-xl bg-slate-950/70 border border-slate-800 flex items-center justify-between text-xs"
    >
      <div class="flex items-center space-x-2">
        <span class="text-cyan-400 font-bold text-[11px]">分路限制:</span>
        <select
          :value="node.params?.position || 'TOP'"
          @change="updateParams({ position: ($event.target as HTMLSelectElement).value })"
          class="bg-slate-900 border border-slate-800 rounded px-2 py-0.5 text-xs text-slate-300 outline-none"
        >
          <option value="TOP">上路 (TOP)</option>
          <option value="JUNGLE">打野 (JUNGLE)</option>
          <option value="MIDDLE">中路 (MIDDLE)</option>
          <option value="BOTTOM">下路 (BOTTOM)</option>
          <option value="UTILITY">辅助 (UTILITY)</option>
        </select>
      </div>
      <button @click="$emit('remove', nodeId)" class="hover:text-rose-400 text-slate-500">
        <X class="w-3.5 h-3.5" />
      </button>
    </div>

    <!-- Leaf: Metric Between -->
    <div
      v-else-if="node.type === 'numberBetween'"
      class="p-2 px-3 rounded-xl bg-slate-950/70 border border-slate-800 flex items-center justify-between text-xs flex-wrap gap-2"
    >
      <div class="flex items-center space-x-2">
        <span class="text-purple-400 font-bold text-[11px]">指标区间:</span>
        <select
          :value="node.params?.metric || 'kills'"
          @change="updateParams({ metric: ($event.target as HTMLSelectElement).value })"
          class="bg-slate-900 border border-slate-800 rounded px-2 py-0.5 text-xs text-slate-300 outline-none"
        >
          <option value="kills">击杀数 (Kills)</option>
          <option value="deaths">死亡数 (Deaths)</option>
          <option value="assists">助攻数 (Assists)</option>
          <option value="kda">KDA 比率</option>
          <option value="damage">英雄总伤害</option>
          <option value="damageTaken">总承受伤害</option>
          <option value="gold">金币总获取</option>
          <option value="cs">补兵总数 (CS)</option>
        </select>
      </div>

      <div class="flex items-center space-x-1.5">
        <span class="text-slate-500 text-[11px]">介于</span>
        <input
          type="number"
          :value="node.params?.min ?? 0"
          @input="updateParams({ min: Number(($event.target as HTMLInputElement).value) })"
          class="w-16 px-1.5 py-0.5 bg-slate-900 border border-slate-800 rounded text-center text-xs text-slate-200 outline-none"
        />
        <span class="text-slate-500 text-[11px]">至</span>
        <input
          type="number"
          :value="node.params?.max ?? 100"
          @input="updateParams({ max: Number(($event.target as HTMLInputElement).value) })"
          class="w-16 px-1.5 py-0.5 bg-slate-900 border border-slate-800 rounded text-center text-xs text-slate-200 outline-none"
        />
        <button @click="$emit('remove', nodeId)" class="hover:text-rose-400 text-slate-500 ml-1">
          <X class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Generic Fallback -->
    <div
      v-else
      class="p-2 px-3 rounded-xl bg-slate-950/70 border border-slate-800 flex items-center justify-between text-xs"
    >
      <span class="text-slate-400 font-mono">{{ node.type }} 条件节点</span>
      <button @click="$emit('remove', nodeId)" class="hover:text-rose-400 text-slate-500">
        <X class="w-3.5 h-3.5" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Plus, X } from 'lucide-vue-next'
import { NDropdown } from 'naive-ui'
import type { AstNode } from '../filter-state'
import ChampionIcon from '../../../../match-card/widgets/ChampionIcon.vue'
import { getChampionName, CHAMPION_NAMES_MAP } from '../../../../match-card/utils/resources'

const props = defineProps<{
  nodeId: string
  rootId: string
  nodeMap: Record<string, AstNode>
}>()

const emit = defineEmits<{
  addNode: [payload: { parentId: string; type: AstNode['type'] }]
  remove: [nodeId: string]
  updateParams: [payload: { nodeId: string; params: Record<string, any> }]
}>()

const node = computed(() => props.nodeMap[props.nodeId])

const championEntries = computed(() => CHAMPION_NAMES_MAP)

const addNodeOptions = [
  { label: 'AND 与组合组', key: 'and' },
  { label: 'OR 或组合组', key: 'or' },
  { type: 'divider', key: 'd1' },
  { label: '英雄限制 (isChampion)', key: 'isChampion' },
  { label: '对局胜利 (isWin)', key: 'isWin' },
  { label: '对局失败 (isLoss)', key: 'isLoss' },
  { label: '分路位置 (isPosition)', key: 'isPosition' },
  { label: '数值区间 (KDA/伤害/经济)', key: 'numberBetween' }
]

function handleAddChild(key: string) {
  emit('addNode', { parentId: props.nodeId, type: key as AstNode['type'] })
}

function updateParams(params: Record<string, any>) {
  emit('updateParams', {
    nodeId: props.nodeId,
    params: {
      ...node.value?.params,
      ...params
    }
  })
}
</script>
