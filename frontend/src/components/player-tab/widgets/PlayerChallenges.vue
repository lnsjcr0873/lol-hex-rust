<template>
  <div v-if="items.length > 0" class="p-3.5 rounded-xl bg-slate-900/80 border border-slate-800 text-xs text-slate-300 select-none space-y-2.5 shadow-sm">
    <div class="flex items-center justify-between pb-2 border-b border-slate-800">
      <span class="font-bold text-xs text-slate-100 flex items-center space-x-1.5">
        <Sparkles class="w-3.5 h-3.5 text-cyan-400" />
        <span>{{ isAssetMode ? '召唤师资产与挑战统计' : '挑战成就概览' }}</span>
      </span>
      <span class="text-[10px] text-slate-500 font-mono">{{ isAssetMode ? 'ASSETS' : 'CHALLENGES' }}</span>
    </div>

    <div class="grid grid-cols-2 gap-2 pt-1 font-mono">
      <div
        v-for="item in items"
        :key="item.id"
        class="flex flex-col p-2 rounded-lg bg-slate-950/60 border border-slate-800/80"
      >
        <div class="flex items-center justify-between">
          <span class="text-[11px] text-slate-400 font-sans truncate">{{ item.name }}</span>
          <span v-if="item.subtext" class="text-[10px] text-cyan-400/80 font-mono">{{ item.subtext }}</span>
        </div>
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

const targetIds = [
  CHALLENGES_CHAMPIONS_ID,
  CHALLENGES_CHAMPION_SKINS_ID,
  CHALLENGES_CHROMA_SKINS_ID,
  CHALLENGES_WARD_SKINS_ID,
  CHALLENGES_SUMMONER_ICONS_ID,
  CHALLENGES_EMOTES_ID
]

interface PlayerChallengeItem {
  id: number | string
  currentValue: number | string
  name: string
  subtext?: string
}

const props = defineProps<{
  challengesData?: {
    playerChallenges?: Array<{ id: number; currentValue: number; [key: string]: any }>
    totalPoints?: { current?: number; level?: string; [key: string]: any }
    categoryProgress?: Array<{ category?: string; current?: number; level?: string; [key: string]: any }>
    [key: string]: any
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

const LEVEL_NAME_MAP: Record<string, string> = {
  NONE: '未定级',
  IRON: '黑铁',
  BRONZE: '青铜',
  SILVER: '白银',
  GOLD: '黄金',
  PLATINUM: '铂金',
  EMERALD: '翡翠',
  DIAMOND: '钻石',
  MASTER: '大师',
  GRANDMASTER: '宗师',
  CHALLENGER: '王者'
}

const CATEGORY_NAME_MAP: Record<string, string> = {
  VETERANCY: '进阶专精',
  EXPERTISE: '竞技技艺',
  COLLECTION: '藏品专精',
  TEAMWORK: '团队协作',
  IMAGINATION: '奇思妙想'
}

const isAssetMode = computed(() => {
  if (!props.challengesData?.playerChallenges || !Array.isArray(props.challengesData.playerChallenges)) {
    return false
  }
  return props.challengesData.playerChallenges.some((c) =>
    targetIds.includes(c.id)
  )
})

const items = computed<PlayerChallengeItem[]>(() => {
  if (!props.challengesData) {
    return []
  }

  // 1. Try to load asset challenges
  if (Array.isArray(props.challengesData.playerChallenges) && props.challengesData.playerChallenges.length > 0) {
    const challengeMap = props.challengesData.playerChallenges.reduce((acc, cur) => {
      acc[cur.id] = cur
      return acc
    }, {} as Record<number, { id: number; currentValue: number }>)

    const assetItems = targetIds
      .map((id): PlayerChallengeItem | null => {
        const c = challengeMap[id]
        if (!c) return null
        return {
          id: c.id,
          currentValue: c.currentValue,
          name: CHALLENGE_NAME_MAP[id] || `挑战 #${id}`
        }
      })
      .filter((i): i is PlayerChallengeItem => i !== null)

    if (assetItems.length > 0) {
      return assetItems
    }
  }

  // 2. Fallback to summary player data (totalPoints + categoryProgress)
  const fallbackItems: PlayerChallengeItem[] = []

  if (props.challengesData.totalPoints) {
    const tp = props.challengesData.totalPoints
    const levelStr = tp.level ? (LEVEL_NAME_MAP[tp.level] || tp.level) : ''
    fallbackItems.push({
      id: 'total_points',
      name: '挑战总点数',
      currentValue: tp.current ?? 0,
      subtext: levelStr
    })
  }

  if (Array.isArray(props.challengesData.categoryProgress)) {
    for (const cat of props.challengesData.categoryProgress) {
      if (!cat || !cat.category) continue
      const catName = CATEGORY_NAME_MAP[cat.category] || cat.category
      const levelStr = cat.level ? (LEVEL_NAME_MAP[cat.level] || cat.level) : ''
      fallbackItems.push({
        id: `cat_${cat.category}`,
        name: catName,
        currentValue: cat.current ?? 0,
        subtext: levelStr
      })
    }
  }

  return fallbackItems
})
</script>
