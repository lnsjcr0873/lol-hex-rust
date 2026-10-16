<template>
  <div class="flex items-center space-x-3 select-none">
    <!-- Solo / Duo Card -->
    <div class="flex items-center space-x-2.5 px-3 py-1.5 rounded-xl bg-slate-900/90 border border-slate-800 shadow-sm min-w-[130px]">
      <div
        class="w-8 h-8 rounded-lg flex items-center justify-center font-black text-xs"
        :class="isSoloUnranked
          ? 'bg-slate-800/60 border border-slate-700/50 text-slate-400'
          : 'bg-amber-500/10 border border-amber-500/30 text-amber-400'"
      >
        单
      </div>
      <div class="flex flex-col">
        <template v-if="isSoloUnranked">
          <div class="text-xs font-bold text-slate-400">未定级</div>
          <div class="text-[10px] text-slate-500 font-mono mt-0.5">暂无排位数据</div>
        </template>
        <template v-else>
          <div class="flex items-center space-x-1.5">
            <span class="text-xs font-black text-amber-300">
              {{ formatTierName(profile?.tier_solo) }} {{ isMasterPlus(profile?.tier_solo) ? '' : (profile?.division_solo || '') }}
            </span>
            <span class="text-[11px] font-mono text-slate-300">{{ profile?.lp_solo ?? 0 }} LP</span>
          </div>
          <div class="text-[10px] text-slate-400 font-mono flex items-center space-x-1.5 mt-0.5">
            <span class="text-[#0ac8b9] font-bold">胜率 {{ profile?.win_rate_solo ?? 0 }}%</span>
            <span>·</span>
            <span>{{ profile?.wins_solo ?? 0 }}胜 {{ profile?.losses_solo ?? 0 }}负</span>
          </div>
        </template>
      </div>
    </div>

    <!-- Flex SR Card -->
    <div class="flex items-center space-x-2.5 px-3 py-1.5 rounded-xl bg-slate-900/90 border border-slate-800 shadow-sm min-w-[130px]">
      <div
        class="w-8 h-8 rounded-lg flex items-center justify-center font-black text-xs"
        :class="isFlexUnranked
          ? 'bg-slate-800/60 border border-slate-700/50 text-slate-400'
          : 'bg-sky-500/10 border border-sky-500/30 text-sky-400'"
      >
        组
      </div>
      <div class="flex flex-col">
        <template v-if="isFlexUnranked">
          <div class="text-xs font-bold text-slate-400">未定级</div>
          <div class="text-[10px] text-slate-500 font-mono mt-0.5">暂无排位数据</div>
        </template>
        <template v-else>
          <div class="flex items-center space-x-1.5">
            <span class="text-xs font-black text-sky-300">
              {{ formatTierName(profile?.tier_flex) }} {{ isMasterPlus(profile?.tier_flex) ? '' : (profile?.division_flex || '') }}
            </span>
            <span class="text-[11px] font-mono text-slate-300">{{ profile?.lp_flex ?? 0 }} LP</span>
          </div>
          <div class="text-[10px] text-slate-400 font-mono flex items-center space-x-1.5 mt-0.5">
            <span class="text-[#0ac8b9] font-bold">胜率 {{ profile?.win_rate_flex ?? 0 }}%</span>
            <span>·</span>
            <span>{{ profile?.wins_flex ?? 0 }}胜 {{ profile?.losses_flex ?? 0 }}负</span>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
export function isUnrankedTier(tier?: string | null): boolean {
  if (!tier) return true
  const upper = tier.trim().toUpperCase()
  return upper === 'UNRANKED' || upper === 'NONE' || upper === 'NA' || upper === ''
}

export function formatTierName(tier?: string): string {
  if (!tier || isUnrankedTier(tier)) return '未定级'
  const trimmed = tier.trim().toUpperCase()
  const map: Record<string, string> = {
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
  return map[trimmed] || tier.trim()
}

export function isMasterPlus(tier?: string): boolean {
  if (!tier) return false
  const upper = tier.trim().toUpperCase()
  return upper === 'MASTER' || upper === 'GRANDMASTER' || upper === 'CHALLENGER'
}
</script>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  profile?: any
}>()

const isSoloUnranked = computed(() => isUnrankedTier(props.profile?.tier_solo))
const isFlexUnranked = computed(() => isUnrankedTier(props.profile?.tier_flex))
</script>
