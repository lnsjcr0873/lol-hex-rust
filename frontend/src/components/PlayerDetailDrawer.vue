<template>
  <div
    v-if="player"
    class="fixed inset-y-0 right-0 w-[420px] max-w-[90vw] bg-[#0a0f1d] border-l border-[#1e2c47] shadow-2xl z-40 flex flex-col transition-transform duration-300 select-none backdrop-blur-md"
  >
    <!-- Drawer Header -->
    <div class="p-4 border-b border-[#162238] flex items-center justify-between bg-[#070b14]/90">
      <div class="flex items-center space-x-3 min-w-0 flex-1">
        <div class="relative shrink-0">
          <ChampionIcon
            :champion-id="player.champion_id"
            class="w-12 h-12 rounded-full border-2 border-[#c8aa6e] object-cover"
            round
          />
          <span class="absolute -bottom-1 -right-1 px-1.5 py-0.2 bg-[#090d1a] border border-[#c8aa6e] text-[9px] text-[#c8aa6e] font-bold rounded-full">
            Lv.{{ player.summoner_level }}
          </span>
        </div>
        <div class="min-w-0 flex-1">
          <div class="flex items-center space-x-1.5 truncate">
            <h3 class="font-bold text-slate-100 text-sm tracking-wide truncate">
              {{ player.game_name || (player as any).summoner_name || player.champion_name || '玩家' }}
            </h3>
            <span v-if="player.tag_line" class="text-xs text-slate-400 shrink-0">#{{ player.tag_line }}</span>
          </div>
          <div class="flex items-center space-x-2 mt-0.5">
            <span class="text-xs font-semibold text-[#c8aa6e]">{{ formatTier(player.tier, player.division) }}</span>
            <span v-if="player.tier && player.tier !== 'UNRANKED'" class="text-[11px] text-slate-400 font-mono">{{ player.lp }} LP</span>
          </div>
        </div>
      </div>

      <div class="flex items-center space-x-1 shrink-0 ml-2">
        <button
          v-if="player.puuid"
          @click="$emit('inspect-history', player.puuid)"
          class="px-2 py-1 rounded-lg text-slate-300 hover:text-cyan-300 hover:bg-[#162238] border border-slate-700/60 transition flex items-center space-x-1 text-xs"
          title="跳转至该玩家的完整战绩画像页面"
        >
          <ExternalLink class="w-3.5 h-3.5" />
          <span class="text-[11px] font-semibold">战绩画像</span>
        </button>

        <button
          @click="$emit('close')"
          class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-[#162238] transition"
          title="关闭抽屉"
        >
          <X class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Drawer Content Scroll Area -->
    <div class="flex-1 overflow-y-auto p-4 space-y-4">
      <!-- Tactical Tags -->
      <div class="bg-[#0c1424] p-3 rounded-xl border border-[#162238] space-y-2">
        <div class="flex items-center justify-between">
          <h4 class="text-xs font-medium text-slate-300">21 维战术特征标签</h4>
          <span class="text-[10px] text-cyan-400">实时算法判定</span>
        </div>
        <TagsArea :player="player" />
      </div>

      <!-- Akari Score 9 Dimensions Breakdown -->
      <div v-if="analysis && analysis.count > 0" class="bg-[#0c1424] p-3 rounded-xl border border-[#162238]">
        <AkariScorePopoverContent :score="analysis.akariScore" />
      </div>
      <div v-else class="bg-[#0c1424] p-3 rounded-xl border border-[#162238] text-xs text-slate-500 text-center py-4">
        暂无该玩家近期战力评分样本
      </div>

      <!-- Radar Dimension Analysis -->
      <div class="bg-[#0c1424] p-3 rounded-xl border border-[#162238]">
        <div class="flex items-center justify-between mb-2">
          <h4 class="text-xs font-medium text-slate-300">六维综合战力画像</h4>
          <span class="text-[10px] text-[#0ac8b9]">标准化分值</span>
        </div>
        <div class="flex justify-center">
          <RadarChart :stats="player.radar" :size="200" />
        </div>
      </div>

      <!-- Jungle Pathing Analysis (When Player is Jungler) -->
      <div v-if="isJungler" class="bg-[#0c1424] p-3 rounded-xl border border-[#162238] space-y-2">
        <div class="flex items-center justify-between">
          <h4 class="text-xs font-medium text-slate-300">打野路径与首轮倾向全景</h4>
          <span class="text-[10px] text-indigo-400">前14m时序推断</span>
        </div>
        <JunglePathingContent v-if="jungleStats" :stats="jungleStats" />
        <div v-else class="text-xs text-slate-500 py-3 text-center">
          暂无该打野玩家前14分钟轨迹数据
        </div>
      </div>

      <!-- Recent Games Overview -->
      <div class="bg-[#0c1424] p-3 rounded-xl border border-[#162238] space-y-2">
        <h4 class="text-xs font-medium text-slate-300">
          {{ sampleCount > 0 ? `近期 ${sampleCount} 场竞技表现` : '近期竞技表现' }}
        </h4>
        <div v-if="sampleCount > 0" class="grid grid-cols-2 gap-2 text-xs">
          <div class="bg-[#080d18] p-2 rounded-lg border border-[#141e30]">
            <span class="text-slate-400 text-[11px]">近{{ sampleCount }}场胜率</span>
            <div class="text-base font-bold text-[#0ac8b9] mt-0.5">{{ player.recent_win_rate ?? 0 }}%</div>
          </div>
          <div class="bg-[#080d18] p-2 rounded-lg border border-[#141e30]">
            <span class="text-slate-400 text-[11px]">场均 KDA</span>
            <div class="text-base font-bold text-slate-100 mt-0.5">{{ player.recent_kda || '0.0/0.0/0.0' }}</div>
          </div>
        </div>
        <div v-else class="text-xs text-slate-500 py-2 text-center">
          暂无近期排位样本
        </div>

        <!-- Current Champion Mastery in Recent Games -->
        <div v-if="player.champion_id > 0" class="bg-[#080d18] p-2.5 rounded-lg border border-[#141e30] flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <ChampionIcon :champion-id="player.champion_id" class="w-8 h-8 rounded border border-[#1e293b]" />
            <div>
              <div class="text-xs text-slate-200">{{ player.champion_name }}</div>
              <div class="text-[10px] text-slate-400">近 {{ sampleCount }} 场出场 {{ player.champ_recent_games ?? 0 }} 场</div>
            </div>
          </div>
          <div class="text-right">
            <div class="text-xs font-bold" :class="(player.champ_recent_win_rate ?? 0) >= 60 ? 'text-[#0ac8b9]' : 'text-slate-300'">
              {{ player.champ_recent_win_rate ?? 0 }}% 胜率
            </div>
          </div>
        </div>
      </div>

      <!-- Recent Matches dots -->
      <div v-if="player.recent_matches && player.recent_matches.length > 0" class="bg-[#0c1424] p-3 rounded-xl border border-[#162238] space-y-2">
        <h4 class="text-xs font-medium text-slate-300">近 {{ player.recent_matches.length }} 场走势</h4>
        <div class="flex items-center space-x-2">
          <div
            v-for="(res, idx) in player.recent_matches"
            :key="idx"
            class="flex-1 py-1.5 rounded flex items-center justify-center font-bold text-xs"
            :class="res === 'win' ? 'bg-blue-900/60 text-blue-300 border border-blue-500/40' : 'bg-red-950/60 text-red-400 border border-red-600/40'"
          >
            {{ res === 'win' ? '胜' : '负' }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { X, ExternalLink } from 'lucide-vue-next'
import type { EnhancedPlayerCard } from './ongoing-game-panel/types'
import RadarChart from './RadarChart.vue'
import ChampionIcon from './widgets/ChampionIcon.vue'
import { AkariScorePopoverContent } from './akari-score'
import { JunglePathingContent } from './jungle-pathing-analysis'
import TagsArea from './ongoing-game-panel/widgets/player-info-card/player-card-tags/TagsArea.vue'
import { adaptPlayerCardToAggregatedAnalysis, createSynthesizedJungleAnalysis } from './ongoing-game-panel/utils/player-adapter'

const props = defineProps<{
  player: EnhancedPlayerCard | null
}>()

defineEmits<{
  (e: 'close'): void
  (e: 'inspect-history', puuid: string): void
}>()

const formatTier = (tier?: string, div?: string) => {
  if (!tier || tier === 'UNRANKED' || tier === 'NONE') return '未定级'
  const tierMap: Record<string, string> = {
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
  const translated = tierMap[tier.toUpperCase()] || tier
  if (!div || div === 'NA' || div === 'NONE' || div.trim() === '') {
    return translated
  }
  const cleanDiv = div.replace(translated, '').replace(new RegExp(`^${tier}\\s*`, 'i'), '').trim()
  return cleanDiv ? `${translated} ${cleanDiv}` : translated
}

const sampleCount = computed<number>(() => {
  if (props.player?.recent_games_count !== undefined && props.player.recent_games_count > 0) {
    return props.player.recent_games_count
  }
  if (props.player?.recent_matches && props.player.recent_matches.length > 0) {
    return props.player.recent_matches.length
  }
  return 0
})

const analysis = computed(() => {
  if (!props.player) return null as any
  return adaptPlayerCardToAggregatedAnalysis(props.player)
})

const isJungler = computed(() => {
  if (!props.player) return false
  const pos = props.player.assigned_position?.toUpperCase()
  return pos === 'JUNGLE' || pos === 'JUG' || props.player.spell1_id === 11 || props.player.spell2_id === 11
})

const jungleStats = computed(() => {
  if (!props.player) return null as any
  return createSynthesizedJungleAnalysis(props.player)
})
</script>
