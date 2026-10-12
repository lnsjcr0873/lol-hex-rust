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
            <h3 class="font-bold text-slate-100 text-sm tracking-wide truncate">{{ player.game_name }}</h3>
            <span class="text-xs text-slate-400 shrink-0">#{{ player.tag_line }}</span>
          </div>
          <div class="flex items-center space-x-2 mt-0.5">
            <span class="text-xs font-semibold text-[#c8aa6e]">{{ player.tier }} {{ player.division }}</span>
            <span class="text-[11px] text-slate-400 font-mono">{{ player.lp }} LP</span>
          </div>
        </div>
      </div>

      <button
        @click="$emit('close')"
        class="p-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-[#162238] transition shrink-0 ml-2"
        title="关闭抽屉"
      >
        <X class="w-4 h-4" />
      </button>
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
      <div class="bg-[#0c1424] p-3 rounded-xl border border-[#162238]">
        <AkariScorePopoverContent :score="analysis.akariScore" />
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
        <JunglePathingContent :stats="jungleStats" />
      </div>

      <!-- Recent 20 Games Overview -->
      <div class="bg-[#0c1424] p-3 rounded-xl border border-[#162238] space-y-2">
        <h4 class="text-xs font-medium text-slate-300">近期 20 场竞技表现</h4>
        <div class="grid grid-cols-2 gap-2 text-xs">
          <div class="bg-[#080d18] p-2 rounded-lg border border-[#141e30]">
            <span class="text-slate-400 text-[11px]">近20场胜率</span>
            <div class="text-base font-bold text-[#0ac8b9] mt-0.5">{{ player.recent_win_rate }}%</div>
          </div>
          <div class="bg-[#080d18] p-2 rounded-lg border border-[#141e30]">
            <span class="text-slate-400 text-[11px]">场均 KDA</span>
            <div class="text-base font-bold text-slate-100 mt-0.5">{{ player.recent_kda }}</div>
          </div>
        </div>

        <!-- Current Champion Mastery in Recent Games -->
        <div class="bg-[#080d18] p-2.5 rounded-lg border border-[#141e30] flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <ChampionIcon :champion-id="player.champion_id" class="w-8 h-8 rounded border border-[#1e293b]" />
            <div>
              <div class="text-xs text-slate-200">{{ player.champion_name }}</div>
              <div class="text-[10px] text-slate-400">近 20 场出场 {{ player.champ_recent_games }} 场</div>
            </div>
          </div>
          <div class="text-right">
            <div class="text-xs font-bold" :class="player.champ_recent_win_rate >= 60 ? 'text-[#0ac8b9]' : 'text-slate-300'">
              {{ player.champ_recent_win_rate }}% 胜率
            </div>
          </div>
        </div>
      </div>

      <!-- Recent 5 Matches dots -->
      <div class="bg-[#0c1424] p-3 rounded-xl border border-[#162238] space-y-2">
        <h4 class="text-xs font-medium text-slate-300">近 5 场走势</h4>
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
import { X } from 'lucide-vue-next'
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
}>()

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
