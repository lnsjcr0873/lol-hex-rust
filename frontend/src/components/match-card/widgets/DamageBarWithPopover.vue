<template>
  <NPopover :delay="100" :show-arrow="false" placement="top">
    <template #trigger>
      <div class="flex flex-col items-center text-[11px] cursor-pointer group">
        <div class="text-[11px] font-mono text-slate-200 group-hover:text-amber-300 transition-colors">
          {{ formatNumber(totalDamage) }}
        </div>
        <DamageBar
          :physical-damage="physicalDamage"
          :magic-damage="magicDamage"
          :true-damage="trueDamage"
          :baseline-damage="baselineDamage"
          :width="width"
          :height="height"
        />
      </div>
    </template>
    <div class="p-2 min-w-52 text-xs bg-slate-900 border border-slate-700 rounded-lg text-slate-200 shadow-xl">
      <div class="flex items-center justify-between mb-2">
        <span class="font-bold text-slate-100">{{ title || '伤害统计' }}</span>
        <span class="text-xs font-mono text-[#0ac8b9]">
          {{ ((totalDamage / Math.max(baselineDamage || 1, 1)) * 100).toFixed(1) }}% 占比
        </span>
      </div>

      <div class="mb-2">
        <DamageBar
          :physical-damage="physicalDamage"
          :magic-damage="magicDamage"
          :true-damage="trueDamage"
          :baseline-damage="baselineDamage"
          :width="180"
          :height="8"
          :border-radius="4"
        />
      </div>

      <div class="grid grid-cols-2 gap-2 pt-2 border-t border-slate-800 text-[11px] font-mono">
        <div>
          <span class="text-slate-400">总量: </span>
          <span class="font-bold text-white">{{ totalDamage.toLocaleString() }}</span>
        </div>
        <div>
          <span class="text-orange-400">物理: </span>
          <span>{{ physicalDamage.toLocaleString() }} ({{ ((physicalDamage / Math.max(totalDamage || 1, 1)) * 100).toFixed(0) }}%)</span>
        </div>
        <div>
          <span class="text-sky-400">魔法: </span>
          <span>{{ magicDamage.toLocaleString() }} ({{ ((magicDamage / Math.max(totalDamage || 1, 1)) * 100).toFixed(0) }}%)</span>
        </div>
        <div>
          <span class="text-slate-200">真实: </span>
          <span>{{ trueDamage.toLocaleString() }} ({{ ((trueDamage / Math.max(totalDamage || 1, 1)) * 100).toFixed(0) }}%)</span>
        </div>
      </div>
    </div>
  </NPopover>
</template>

<script setup lang="ts">
import { NPopover } from 'naive-ui'
import DamageBar from './DamageBar.vue'

withDefaults(
  defineProps<{
    title?: string
    physicalDamage?: number
    magicDamage?: number
    trueDamage?: number
    totalDamage?: number
    baselineDamage?: number
    width?: number
    height?: number
  }>(),
  {
    title: '',
    physicalDamage: 0,
    magicDamage: 0,
    trueDamage: 0,
    totalDamage: 0,
    baselineDamage: 1,
    width: 52,
    height: 6
  }
)

function formatNumber(num: number): string {
  if (num >= 1000) {
    return `${(num / 1000).toFixed(1)}k`
  }
  return num.toString()
}
</script>
