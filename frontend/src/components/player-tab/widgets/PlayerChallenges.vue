<template>
  <div v-if="items.length > 0" class="p-3.5 rounded-xl bg-slate-900/80 border border-slate-800 text-xs text-slate-300 select-none space-y-2.5 shadow-sm">
    <div class="flex items-center justify-between pb-2 border-b border-slate-800">
      <span class="font-bold text-xs text-slate-100 flex items-center space-x-1.5">
        <Sparkles class="w-3.5 h-3.5 text-cyan-400" />
        <span>召唤师资产与挑战统计</span>
      </span>
      <span class="text-[10px] text-slate-500 font-mono">ASSETS</span>
    </div>

    <div class="grid grid-cols-2 gap-2 pt-1 font-mono">
      <div
        v-for="item in items"
        :key="item.id"
        class="flex flex-col p-2 rounded-lg bg-slate-950/60 border border-slate-800/80"
      >
        <span class="text-[11px] text-slate-400 font-sans truncate">{{ item.name }}</span>
        <span class="text-sm font-bold text-slate-100 mt-0.5">{{ item.currentValue }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Sparkles } from 'lucide-vue-next'

const CHALLENGES_CHAMPIONS_ID = 505001
const CHALLENGES_CHAMPION_SKINS_ID = 510001
const CHALLENGES_CHROMA_SKINS_ID = 510011
const CHALLENGES_WARD_SKINS_ID = 504003
const CHALLENGES_SUMMONER_ICONS_ID = 504002
const CHALLENGES_EMOTES_ID = 504004

interface PlayerChallengeItem {
  id: number
  currentValue: number
  name: string
}

const props = defineProps<{
  challengesData?: {
    playerChallenges?: Array<{ id: number; currentValue: number; [key: string]: any }>
  } | null
}>()

const CHALLENGE_NAME_MAP: Record<number, string> = {
  [CHALLENGES_CHAMPIONS_ID]: '拥有英雄',
  [CHALLENGES_CHAMPION_SKINS_ID]: '拥有皮肤',
  [CHALLENGES_CHROMA_SKINS_ID]: '炫彩皮肤',
  [CHALLENGES_WARD_SKINS_ID]: '守卫皮肤',
  [CHALLENGES_SUMMONER_ICONS_ID]: '召唤师头像',
  [CHALLENGES_EMOTES_ID]: '表情数量'
}

const items = computed<PlayerChallengeItem[]>(() => {
  if (!props.challengesData?.playerChallenges || !Array.isArray(props.challengesData.playerChallenges)) {
    return []
  }

  const challengeMap = props.challengesData.playerChallenges.reduce((acc, cur) => {
    acc[cur.id] = cur
    return acc
  }, {} as Record<number, { id: number; currentValue: number }>)

  const targetIds = [
    CHALLENGES_CHAMPIONS_ID,
    CHALLENGES_CHAMPION_SKINS_ID,
    CHALLENGES_CHROMA_SKINS_ID,
    CHALLENGES_WARD_SKINS_ID,
    CHALLENGES_SUMMONER_ICONS_ID,
    CHALLENGES_EMOTES_ID
  ]

  return targetIds
    .map((id) => {
      const c = challengeMap[id]
      if (!c) return null
      return {
        id: c.id,
        currentValue: c.currentValue,
        name: CHALLENGE_NAME_MAP[id] || `挑战 #${id}`
      }
    })
    .filter((i): i is PlayerChallengeItem => i !== null)
})
</script>
