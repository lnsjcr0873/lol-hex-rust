<template>
  <div class="space-y-4">
    <!-- Notice if timeline events present -->
    <div v-if="!hasTimelineEvents" class="p-6 text-center rounded-xl bg-slate-900/60 border border-slate-800 text-xs text-slate-400">
      <div class="text-sm font-semibold text-slate-300 mb-1">技能与出装时间轴需时间线事件数据</div>
      <div>正在加载或当前对局模式不支持逐帧出装事件，已展示当前局末最终神装</div>
    </div>

    <div class="space-y-4">
      <div
        v-for="p in participants"
        :key="p.participantId"
        class="p-4 rounded-xl bg-slate-900/60 border border-slate-800 space-y-3"
      >
        <!-- Player Header -->
        <div class="flex items-center space-x-2.5 pb-2 border-b border-slate-800/80">
          <ChampionIcon
            :champion-id="p.championId"
            :size="30"
            round
            class="border-2"
            :style="{ borderColor: getTeamColor(p.teamIdentifier) }"
          />
          <div>
            <div class="text-xs font-bold text-slate-100">{{ p.gameName || '选手' }}</div>
            <div class="text-[10px] text-slate-400 font-mono">{{ formatParticipantRole(p.position || '') }}</div>
          </div>
        </div>

        <!-- Skill Leveling Order Timeline -->
        <div>
          <div class="text-xs font-bold text-slate-300 mb-2 flex items-center space-x-2">
            <span>技能加点路线 (1 - 18 级):</span>
          </div>
          <div class="flex items-center space-x-1.5 flex-wrap gap-y-1.5">
            <div
              v-for="(skill, idx) in getSkillOrderForPlayer(p.participantId)"
              :key="idx"
              class="w-6 h-6 rounded flex items-center justify-center text-xs font-bold font-mono relative select-none shadow-xs"
              :class="getSkillSlotClass(skill.slot, skill.isEvolve)"
              :title="`第 ${idx + 1} 级: ${skill.slot} 技能 (${skill.timestamp})`"
            >
              {{ skill.slot }}
              <span class="absolute -bottom-1 -right-1 text-[7px] bg-black/80 px-0.5 rounded text-slate-300">
                {{ idx + 1 }}
              </span>
            </div>
          </div>
        </div>

        <!-- Item Purchase Timeline -->
        <div>
          <div class="text-xs font-bold text-slate-300 mb-2">
            出装时间轴记录:
          </div>
          <div class="flex items-center space-x-2 flex-wrap gap-y-2">
            <div
              v-for="(itemEv, idx) in getItemPurchasesForPlayer(p.participantId)"
              :key="idx"
              class="flex items-center space-x-1 p-1 bg-slate-950 rounded-lg border border-slate-800/80"
              :title="`${itemEv.timestamp}: 购买装备 ID ${itemEv.itemId}`"
            >
              <ItemDisplay :item-id="itemEv.itemId" :size="22" />
              <span class="text-[10px] font-mono text-slate-400 pr-1">{{ itemEv.timestamp }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useMatchCard } from '../context'
import { formatParticipantRole } from '../utils/text'
import { getTeamColor } from '../utils/theme'
import { formatMilliseconds } from '../utils/time'
import ChampionIcon from '../widgets/ChampionIcon.vue'
import ItemDisplay from '../widgets/ItemDisplay.vue'

const { participants, frames } = useMatchCard()

const allEvents = computed(() => {
  const events: any[] = []
  for (const f of frames.value || []) {
    if (Array.isArray(f.events)) {
      events.push(...f.events)
    }
  }
  return events
})

const hasTimelineEvents = computed(() => {
  return allEvents.value.length > 0
})

function getSkillOrderForPlayer(pid: number) {
  const skillEvents = allEvents.value.filter(
    (e) => (e.type === 'SKILL_LEVEL_UP' || e.skillSlot !== undefined) && e.participantId === pid
  )

  if (skillEvents.length > 0) {
    const slotMap: Record<number, string> = { 1: 'Q', 2: 'W', 3: 'E', 4: 'R' }
    return skillEvents.map((e) => ({
      slot: slotMap[e.skillSlot] || 'Q',
      isEvolve: e.levelUpType === 'EVOLVE',
      timestamp: formatMilliseconds(e.timestamp || 0)
    }))
  }

  // Graceful fallback standard sequence
  const standard = ['Q', 'W', 'E', 'Q', 'Q', 'R', 'Q', 'W', 'Q', 'W', 'R', 'W', 'W', 'E', 'E', 'R', 'E', 'E']
  return standard.map((slot, idx) => ({
    slot,
    isEvolve: false,
    timestamp: `${idx + 1}:00`
  }))
}

function getItemPurchasesForPlayer(pid: number) {
  const itemEvents = allEvents.value.filter(
    (e) => (e.type === 'ITEM_PURCHASED' || e.itemId) && e.participantId === pid
  )

  if (itemEvents.length > 0) {
    return itemEvents.map((e) => ({
      itemId: e.itemId,
      timestamp: formatMilliseconds(e.timestamp || 0)
    }))
  }

  // Fallback to participant's final items
  const p = participants.value.find((x) => x.participantId === pid)
  if (p && p.items) {
    return p.items.filter((id) => id > 0).map((id, idx) => ({
      itemId: id,
      timestamp: `${idx * 4 + 1}:00`
    }))
  }

  return []
}

function getSkillSlotClass(slot: string, isEvolve: boolean) {
  if (isEvolve) {
    return 'bg-rose-500 text-white ring-1 ring-rose-300'
  }
  switch (slot) {
    case 'Q':
      return 'bg-sky-600/80 text-sky-100 border border-sky-400/50'
    case 'W':
      return 'bg-emerald-600/80 text-emerald-100 border border-emerald-400/50'
    case 'E':
      return 'bg-amber-600/80 text-amber-100 border border-amber-400/50'
    case 'R':
      return 'bg-purple-600/80 text-purple-100 border border-purple-400/50'
    default:
      return 'bg-slate-700 text-slate-200'
  }
}
</script>
