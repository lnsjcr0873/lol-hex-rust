<template>
  <n-modal
    :show="show"
    @update:show="$emit('update:show', $event)"
    transform-origin="center"
  >
    <div class="w-[680px] max-w-[92vw] h-[80vh] max-h-[720px] flex flex-col rounded-2xl bg-slate-900/95 border border-slate-700/80 backdrop-blur-xl shadow-2xl p-5 text-slate-200 select-none overflow-hidden">
      <!-- Modal Header -->
      <div class="flex items-center justify-between pb-3.5 border-b border-slate-800 shrink-0 flex-wrap gap-2">
        <div class="flex items-center space-x-2">
          <Filter class="w-5 h-5 text-cyan-400" />
          <h2 class="text-base font-bold text-slate-100">战绩高级筛选器</h2>
        </div>

        <div class="flex items-center space-x-2">
          <!-- Simple / Advanced Mode Toggle -->
          <div class="flex items-center bg-slate-950 p-0.5 rounded-xl border border-slate-800 text-xs">
            <button
              @click="$emit('update:activeMode', 'simple')"
              class="px-2.5 py-1 rounded-lg font-medium transition text-[11px]"
              :class="activeMode === 'simple' ? 'bg-cyan-950 text-cyan-300 font-semibold shadow-sm' : 'text-slate-400 hover:text-slate-200'"
            >
              简易筛选
            </button>
            <button
              @click="$emit('update:activeMode', 'advanced')"
              class="px-2.5 py-1 rounded-lg font-medium transition text-[11px]"
              :class="activeMode === 'advanced' ? 'bg-cyan-950 text-cyan-300 font-semibold shadow-sm' : 'text-slate-400 hover:text-slate-200'"
            >
              AST 树状高级筛选
            </button>
          </div>

          <!-- Reset Button -->
          <button
            @click="$emit('reset')"
            class="px-2.5 py-1 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-rose-300 border border-slate-700/80 text-[11px] font-semibold transition active:scale-95 flex items-center space-x-1"
            title="清空并重置全部筛选条件"
          >
            <RotateCcw class="w-3.5 h-3.5" />
            <span>重置</span>
          </button>

          <!-- Collect Mode Button -->
          <button
            @click="showCollectSettings = true"
            :disabled="isCollecting"
            class="px-3 py-1 rounded-xl bg-cyan-600 hover:bg-cyan-500 disabled:opacity-40 text-white font-bold text-[11px] transition active:scale-95 flex items-center space-x-1.5 shadow-sm"
            title="开启后台深度自动翻页收集"
          >
            <Sparkles class="w-3.5 h-3.5" />
            <span>深度收集模式</span>
          </button>

          <!-- Close Button -->
          <button
            @click="$emit('update:show', false)"
            class="p-1 rounded-lg hover:bg-slate-800 text-slate-400 hover:text-white"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </div>

      <!-- Main Body -->
      <div class="flex-1 overflow-y-auto py-3 pr-1">
        <SimpleMatchHistoryFilter
          v-if="activeMode === 'simple'"
          :model-value="simpleState"
          @update:model-value="$emit('update:simpleState', $event)"
        />

        <AdvancedMatchHistoryFilter
          v-else
          :model-value="advancedState"
          @update:model-value="$emit('update:advancedState', $event)"
        />
      </div>

      <!-- Footer -->
      <div class="pt-3 border-t border-slate-800 flex items-center justify-between shrink-0 text-xs text-slate-400">
        <span class="text-[11px]">
          筛选器由 HexBrain 原生谓词解析器提供支撑，支持全指标与复合逻辑过滤
        </span>
        <button
          @click="$emit('update:show', false)"
          class="px-4 py-1.5 rounded-xl bg-cyan-600 hover:bg-cyan-500 text-white font-bold transition active:scale-95 text-xs shadow"
        >
          应用并查看结果
        </button>
      </div>
    </div>

    <!-- Collect Mode Settings Dialog -->
    <n-modal v-model:show="showCollectSettings" transform-origin="center">
      <div class="w-[420px] max-w-[90vw] p-5 rounded-2xl bg-slate-900 border border-slate-700 shadow-2xl text-slate-200 space-y-4">
        <div class="flex items-center justify-between pb-2 border-b border-slate-800">
          <div class="flex items-center space-x-2">
            <Sparkles class="w-4 h-4 text-cyan-400" />
            <h3 class="text-sm font-bold text-slate-100">深度收集模式配置</h3>
          </div>
          <button @click="showCollectSettings = false" class="text-slate-400 hover:text-white">
            <X class="w-4 h-4" />
          </button>
        </div>

        <div class="space-y-3 text-xs">
          <p class="text-slate-400 leading-relaxed text-[11px]">
            收集模式将根据当前筛选条件自动向后静默连续扫描多页战绩，直至达到期望命中场数或最大轮次。
          </p>

          <div class="flex items-center justify-between">
            <span class="text-slate-300">单次翻页抓取数量</span>
            <input
              v-model.number="collectConfig.countPerIteration"
              type="number"
              min="10"
              max="200"
              step="10"
              class="w-24 px-2 py-1 rounded-lg bg-slate-950 border border-slate-800 text-right text-xs text-slate-200 outline-none"
            />
          </div>

          <div class="flex items-center justify-between">
            <span class="text-slate-300">期望命中目标场数</span>
            <input
              v-model.number="collectConfig.expectedCount"
              type="number"
              min="5"
              max="500"
              step="5"
              class="w-24 px-2 py-1 rounded-lg bg-slate-950 border border-slate-800 text-right text-xs text-slate-200 outline-none"
            />
          </div>

          <div class="flex items-center justify-between">
            <span class="text-slate-300">最大静默扫描轮数</span>
            <input
              v-model.number="collectConfig.maxIteration"
              type="number"
              min="1"
              max="50"
              class="w-24 px-2 py-1 rounded-lg bg-slate-950 border border-slate-800 text-right text-xs text-slate-200 outline-none"
            />
          </div>
        </div>

        <div class="flex items-center justify-end space-x-2 pt-2 border-t border-slate-800">
          <button
            @click="showCollectSettings = false"
            class="px-3 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-semibold"
          >
            取消
          </button>
          <button
            @click="handleStartCollect"
            class="px-4 py-1.5 rounded-xl bg-cyan-600 hover:bg-cyan-500 text-white text-xs font-bold transition active:scale-95 shadow-md flex items-center space-x-1"
          >
            <Sparkles class="w-3.5 h-3.5" />
            <span>开始深度收集</span>
          </button>
        </div>
      </div>
    </n-modal>
  </n-modal>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { NModal } from 'naive-ui'
import { Filter, X, RotateCcw, Sparkles } from 'lucide-vue-next'
import type { SimpleMatchHistoryFilterState, AdvancedFilterState } from './filter-state'
import SimpleMatchHistoryFilter from './SimpleMatchHistoryFilter.vue'
import AdvancedMatchHistoryFilter from './AdvancedMatchHistoryFilter.vue'

const props = defineProps<{
  show: boolean
  simpleState: SimpleMatchHistoryFilterState
  advancedState: AdvancedFilterState
  activeMode: 'simple' | 'advanced'
  isCollecting?: boolean
}>()

const emit = defineEmits<{
  'update:show': [val: boolean]
  'update:simpleState': [val: SimpleMatchHistoryFilterState]
  'update:advancedState': [val: AdvancedFilterState]
  'update:activeMode': [mode: 'simple' | 'advanced']
  reset: []
  startCollect: [options: { countPerIteration: number; expectedCount: number; maxIteration: number }]
}>()

const showCollectSettings = ref(false)
const collectConfig = ref({
  countPerIteration: 20,
  expectedCount: 20,
  maxIteration: 10
})

function handleStartCollect() {
  showCollectSettings.value = false
  emit('update:show', false)
  emit('startCollect', { ...collectConfig.value })
}
</script>
