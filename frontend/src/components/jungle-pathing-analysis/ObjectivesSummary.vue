<template>
  <div>
    <div class="mb-1 font-bold text-slate-100">
      史诗级野怪控制
    </div>
    <div class="grid grid-cols-2 gap-x-2 gap-y-1 text-[11px]">
      <div class="inline-flex min-w-0 items-center gap-1.5 text-slate-300">
        <DragonIcon class="w-3.5 h-3.5 text-blue-400 shrink-0" />
        <span class="truncate text-slate-400">控一龙率</span>
        <span class="font-semibold text-slate-100 font-mono">{{ Math.round((stats.objectives?.firstDragonRate ?? 0) * 100) }}%</span>
      </div>

      <div class="inline-flex min-w-0 items-center gap-1.5 text-slate-300">
        <DragonIcon class="w-3.5 h-3.5 text-cyan-400 shrink-0" />
        <span class="truncate text-slate-400">单挑小龙率</span>
        <span class="font-semibold text-slate-100 font-mono">{{ Math.round((stats.objectives?.soloDragonRate ?? 0) * 100) }}%</span>
      </div>

      <div class="inline-flex min-w-0 items-center gap-1.5 text-slate-300">
        <DragonIcon class="w-3.5 h-3.5 text-amber-400 shrink-0" />
        <span class="truncate text-slate-400">场均小龙</span>
        <span class="font-semibold text-slate-100 font-mono">{{ roundToTenth(stats.objectives?.avgDragons ?? 0) }}</span>
        <span v-if="stats.objectives?.avgFirstDragonTime != null" class="text-slate-500 text-[10px]">
          ({{ formatTime(stats.objectives.avgFirstDragonTime) }})
        </span>
      </div>

      <div class="inline-flex min-w-0 items-center gap-1.5 text-slate-300">
        <VoidGrubIcon class="w-3.5 h-3.5 text-purple-400 shrink-0" />
        <span class="truncate text-slate-400">场均虚空巢虫</span>
        <span class="font-semibold text-slate-100 font-mono">{{ roundToTenth(stats.objectives?.avgVoidgrubs ?? 0) }}</span>
        <span v-if="stats.objectives?.avgFirstVoidgrubTime != null" class="text-slate-500 text-[10px]">
          ({{ formatTime(stats.objectives.avgFirstVoidgrubTime) }})
        </span>
      </div>

      <div class="inline-flex min-w-0 items-center gap-1.5 text-slate-300">
        <RiftHeraldIcon class="w-3.5 h-3.5 text-emerald-400 shrink-0" />
        <span class="truncate text-slate-400">场均先锋</span>
        <span class="font-semibold text-slate-100 font-mono">{{ roundToTenth(stats.objectives?.avgHeralds ?? 0) }}</span>
        <span v-if="stats.objectives?.avgFirstHeraldTime != null" class="text-slate-500 text-[10px]">
          ({{ formatTime(stats.objectives.avgFirstHeraldTime) }})
        </span>
      </div>

      <div class="inline-flex min-w-0 items-center gap-1.5 text-slate-300">
        <BaronIcon class="w-3.5 h-3.5 text-rose-400 shrink-0" />
        <span class="truncate text-slate-400">场均男爵</span>
        <span class="font-semibold text-slate-100 font-mono">{{ roundToTenth(stats.objectives?.avgBarons ?? 0) }}</span>
        <span v-if="stats.objectives?.avgFirstBaronTime != null" class="text-slate-500 text-[10px]">
          ({{ formatTime(stats.objectives.avgFirstBaronTime) }})
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { AggregatedJungleAnalysis } from '../../shared/data-adapter/analysis/player'
import BaronIcon from '../icons/BaronIcon.vue'
import DragonIcon from '../icons/DragonIcon.vue'
import RiftHeraldIcon from '../icons/RiftHeraldIcon.vue'
import VoidGrubIcon from '../icons/VoidGrubIcon.vue'
import { formatTime, roundToTenth } from './format'

defineProps<{
  stats: AggregatedJungleAnalysis
}>()
</script>
