<template>
  <div
    v-if="active"
    class="fixed top-16 right-8 z-40 p-3 px-4 rounded-2xl bg-slate-900/95 border border-cyan-500/60 shadow-2xl backdrop-blur-xl flex items-center space-x-4 select-none animate-in fade-in slide-in-from-top-2 duration-200"
  >
    <div class="flex items-center space-x-3">
      <div class="relative flex items-center justify-center">
        <Loader2 class="w-6 h-6 text-cyan-400 animate-spin" />
      </div>

      <div class="space-y-0.5 text-left">
        <div class="text-xs font-bold text-slate-100 flex items-center space-x-1.5">
          <span>深度战绩收集扫描中</span>
          <span class="px-1.5 py-0.2 rounded bg-cyan-950 text-cyan-300 font-mono text-[9px] border border-cyan-500/40">
            静默翻页
          </span>
        </div>
        <div class="text-[11px] text-slate-400 font-mono">
          已命中: <span class="text-cyan-300 font-bold">{{ matchedCount }}</span> / {{ expectedCount }} 场
          <span class="mx-1 text-slate-600">·</span>
          进度: 第 <span class="text-slate-200">{{ currentIteration }}</span> / {{ maxIteration }} 轮
        </div>
      </div>
    </div>

    <button
      @click="$emit('stop')"
      class="px-2.5 py-1.5 rounded-xl bg-amber-950/80 hover:bg-amber-900 border border-amber-600 text-amber-200 text-xs font-bold transition active:scale-95 flex items-center space-x-1 shrink-0 shadow-sm"
      title="立即停止后台收集并查看当前已匹配对局"
    >
      <Square class="w-3 h-3 fill-amber-300" />
      <span>停止</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { Loader2, Square } from 'lucide-vue-next'

defineProps<{
  active: boolean
  currentIteration: number
  maxIteration: number
  matchedCount: number
  expectedCount: number
}>()

defineEmits<{
  stop: []
}>()
</script>
