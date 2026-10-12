<template>
  <div class="p-3 bg-slate-900 border border-slate-700 rounded-xl text-slate-100 min-w-72 max-w-sm shadow-2xl">
    <div class="flex items-center justify-between pb-2 border-b border-slate-800 text-xs font-bold mb-2">
      <span class="text-rose-400">受击伤害来源明细</span>
      <span class="font-mono text-slate-400">总计: {{ totalDamageReceived.toLocaleString() }}</span>
    </div>

    <!-- Breakdown items -->
    <div v-if="damageBreakdown.length > 0" class="space-y-2">
      <div
        v-for="(item, idx) in damageBreakdown"
        :key="idx"
        class="flex items-center justify-between text-xs p-1.5 rounded-lg bg-slate-950/60 border border-slate-800/80"
      >
        <div class="flex items-center space-x-2">
          <ChampionIcon :champion-id="item.championId" :size="24" round />
          <span class="font-semibold text-slate-200">{{ item.name }}</span>
        </div>
        <div class="text-right">
          <div class="font-mono font-bold text-white">{{ item.damage.toLocaleString() }}</div>
          <div class="text-[10px] text-slate-400 font-mono">{{ item.percentage }}%</div>
        </div>
      </div>
    </div>
    <div v-else class="text-xs text-slate-500 py-2 text-center">
      无详细技能受击明细（基础击杀记录）
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import ChampionIcon from './ChampionIcon.vue'
import { getChampionName } from '../utils/resources'
import { useMatchCard } from '../context'

const props = defineProps<{
  event: any
}>()

const { participants } = useMatchCard()

const participantMap = computed(() => {
  const map: Record<number, any> = {}
  for (const p of participants.value) {
    map[p.participantId] = p
  }
  return map
})

const totalDamageReceived = computed(() => {
  if (Array.isArray(props.event.victimDamageReceived)) {
    return props.event.victimDamageReceived.reduce((acc: number, d: any) => acc + (d.totalDamage || 0), 0)
  }
  return 1000
})

const damageBreakdown = computed(() => {
  if (Array.isArray(props.event.victimDamageReceived) && props.event.victimDamageReceived.length > 0) {
    const total = Math.max(totalDamageReceived.value, 1)
    return props.event.victimDamageReceived.map((d: any) => {
      const p = participantMap.value[d.participantId]
      const cid = p ? p.championId : 0
      const cname = p ? getChampionName(cid) : (d.spellName || '未知攻击')
      return {
        championId: cid,
        name: cname,
        damage: d.totalDamage || 0,
        percentage: (((d.totalDamage || 0) / total) * 100).toFixed(0)
      }
    })
  }

  // Fallback: estimate from killer
  const killer = participantMap.value[props.event.killerId]
  if (killer) {
    return [
      {
        championId: killer.championId,
        name: getChampionName(killer.championId),
        damage: 1000,
        percentage: '100'
      }
    ]
  }

  return []
})
</script>
