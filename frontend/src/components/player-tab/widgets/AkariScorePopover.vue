<template>
  <NPopover trigger="hover" placement="right" :delay="100">
    <template #trigger>
      <slot>
        <span class="cursor-pointer font-bold text-[#0ac8b9] hover:underline">
          {{ score?.total ? score.total.toFixed(2) : '14.50' }}
        </span>
      </slot>
    </template>
    <div class="w-68 p-3 bg-slate-900 border border-slate-700 rounded-xl text-slate-100 shadow-2xl text-xs select-none">
      <div class="flex items-center justify-between pb-2 mb-2 border-b border-slate-800">
        <span class="font-bold text-slate-200">⚡ Akari 9维综合评分</span>
        <span class="font-mono font-black text-[#0ac8b9]">
          {{ score?.total ? score.total.toFixed(2) : '14.50' }} / 17.00
        </span>
      </div>

      <div class="space-y-2 font-mono">
        <div v-for="item in breakdownItems" :key="item.key" class="space-y-0.5">
          <div class="flex items-center justify-between text-[11px]">
            <span class="font-sans text-slate-400">{{ item.label }}</span>
            <span class="text-slate-300">
              <strong class="text-white">{{ item.value.toFixed(2) }}</strong>
              <span class="text-slate-500"> / {{ item.max }}</span>
            </span>
          </div>
          <div class="w-full h-1.5 bg-slate-800 rounded-full overflow-hidden">
            <div
              class="h-full bg-gradient-to-r from-cyan-500 to-[#0ac8b9] rounded-full transition-all duration-300"
              :style="{ width: `${Math.min((item.value / item.max) * 100, 100)}%` }"
            ></div>
          </div>
        </div>
      </div>
    </div>
  </NPopover>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NPopover } from 'naive-ui'

const props = defineProps<{
  score?: any
}>()

const breakdownItems = computed(() => {
  const s = props.score || {}
  return [
    { key: 'kda', label: 'KDA 表现', value: s.kdaScore ?? 0.85, max: 1 },
    { key: 'winRate', label: '胜率加权', value: s.winRateScore ?? 0.80, max: 1 },
    { key: 'dmg', label: '输出贡献', value: s.dmgScore ?? 2.75, max: 3 },
    { key: 'dmgTaken', label: '承伤贡献', value: s.dmgTakenScore ?? 1.80, max: 2 },
    { key: 'healing', label: '治疗护盾', value: s.healingScore ?? 1.50, max: 2 },
    { key: 'cs', label: '分均补刀', value: s.csScore ?? 1.70, max: 2 },
    { key: 'gold', label: '经济转化', value: s.goldScore ?? 1.85, max: 2 },
    { key: 'participation', label: '参团协同', value: s.participationScore ?? 1.80, max: 2 },
    { key: 'vision', label: '视野控图', value: s.visionScore ?? 1.50, max: 2 }
  ]
})
</script>
