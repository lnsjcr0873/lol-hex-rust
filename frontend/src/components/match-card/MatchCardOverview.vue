<template>
  <div
    v-if="participant"
    class="relative box-border flex w-full overflow-hidden rounded-xl border border-solid select-none transition-all cursor-pointer group"
    :class="[
      cardBorderClass,
      winStyleType === 'win' ? 'bg-[#091526]/90 hover:bg-[#0c1c33]' : (winStyleType === 'loss' ? 'bg-[#180a0e]/90 hover:bg-[#220e14]' : 'bg-[#0f172a]/90 hover:bg-[#162238]')
    ]"
    :style="{ minHeight: `${MATCH_CARD_COLLAPSED_HEIGHT_PX}px` }"
    @click="$emit('toggle-expand')"
  >
    <!-- Left Win/Loss accent indicator strip -->
    <div
      class="w-1.5 self-stretch shrink-0"
      :class="winStyleType === 'win' ? 'bg-blue-500' : (winStyleType === 'loss' ? 'bg-red-500' : 'bg-slate-500')"
    ></div>

    <!-- Main Content Container -->
    <div class="flex-1 p-3 flex flex-col justify-between min-w-0">
      <!-- Top Row: Champion, Spells/Runes, KDA, Damage, CS, Mini 10-player preview -->
      <div class="flex items-center justify-between gap-4">
        <!-- Champion & Spells & Identity -->
        <div class="flex items-center space-x-3 shrink-0">
          <!-- Champion Avatar with Position badge -->
          <div class="relative shrink-0">
            <ChampionIcon
              :champion-id="participant.championId"
              :size="48"
              class="border-2 rounded-xl"
              :class="winStyleType === 'win' ? 'border-blue-500/80 shadow-[0_0_8px_rgba(59,130,246,0.3)]' : (winStyleType === 'loss' ? 'border-red-500/80 shadow-[0_0_8px_rgba(239,68,68,0.3)]' : 'border-slate-500/80')"
            />
            <span
              v-if="participant.position"
              class="absolute -bottom-1 -right-1 p-0.5 rounded-sm bg-black/85 border border-slate-700 text-slate-300"
            >
              <PositionIcon :position="participant.position" :size="12" />
            </span>
          </div>

          <!-- Spells & Runes 2x2 grid -->
          <div class="flex flex-col space-y-1 shrink-0">
            <div class="flex space-x-1">
              <SummonerSpellDisplay :spell-id="participant.spells[0]" :size="18" />
              <SummonerSpellDisplay :spell-id="participant.spells[1]" :size="18" />
            </div>
            <div class="flex space-x-1" v-if="participant.perks">
              <PerkDisplay :perk-id="participant.perks.styles?.[0]?.selections?.[0]?.perk" :size="18" />
              <PerkstyleDisplay :perkstyle-id="participant.perks.styles?.[1]?.style" :size="18" />
            </div>
          </div>

          <!-- Result & Game Name -->
          <div class="flex flex-col">
            <div class="flex items-center space-x-2">
              <span
                class="text-sm font-extrabold uppercase tracking-wide"
                :class="winStyleType === 'win' ? 'text-blue-400' : (winStyleType === 'loss' ? 'text-red-400' : 'text-slate-400')"
              >
                {{ gameResultText }}
              </span>
              <span class="text-xs font-bold text-slate-100 truncate max-w-28">
                {{ getChampionName(participant.championId) }}
              </span>
            </div>
            <div class="text-[11px] text-slate-400 mt-0.5 flex items-center space-x-1.5 font-mono">
              <span>{{ queueName }}</span>
              <span>·</span>
              <span>{{ formatDuration(basicInfo.gameDuration) }}</span>
            </div>
          </div>
        </div>

        <!-- Center: KDA + KP% + Radar Popover trigger -->
        <NPopover trigger="hover" placement="top" :delay="200">
          <template #trigger>
            <div class="flex items-center space-x-6 cursor-help px-3 py-1 rounded-lg bg-slate-950/40 border border-slate-800/60 hover:border-slate-700 transition">
              <!-- KDA Stats -->
              <div class="text-center">
                <div v-if="participant.deaths === 0 && (participant.kills > 0 || participant.assists > 0)" class="text-xs font-black text-amber-300 font-mono">
                  {{ participant.kills }} / <span class="text-red-400">0</span> / {{ participant.assists }}
                  <span class="text-[10px] ml-1 px-1 bg-amber-500/20 text-amber-300 rounded font-sans">完美</span>
                </div>
                <div v-else class="text-xs font-black font-mono text-slate-100">
                  {{ participant.kills }} / <span class="text-red-400">{{ participant.deaths }}</span> / {{ participant.assists }}
                </div>
                <div class="text-[10px] font-mono text-slate-400 mt-0.5">
                  <span class="text-slate-200 font-bold">{{ participant.kda.toFixed(2) }} KDA</span>
                  <span class="text-[#0ac8b9] ml-1">({{ (participant.killParticipation * 100).toFixed(0) }}% 参团)</span>
                </div>
              </div>

              <!-- Damage & CS -->
              <div class="text-right font-mono text-xs hidden sm:block">
                <div class="text-slate-200 font-bold">
                  {{ (participant.totalDamageDealtToChampions / 1000).toFixed(1) }}k 输出
                  <span class="text-[#0ac8b9] text-[10px] ml-0.5">({{ damageSharePct }}%)</span>
                </div>
                <div class="text-[10px] text-slate-400 mt-0.5">
                  {{ participant.cs }} 刀 ({{ ((participant.cs / Math.max(basicInfo.gameDuration / 60, 1))).toFixed(1) }}/分)
                </div>
              </div>
            </div>
          </template>
          <!-- 6-Axis Radar Popover Content -->
          <div class="p-2 bg-slate-900 border border-slate-700 rounded-xl shadow-2xl">
            <div class="text-xs font-bold text-center text-slate-200 mb-2">个人六维能力雷达图</div>
            <RadarChart :puuid="participant.puuid" :size="240" />
          </div>
        </NPopover>

        <!-- Right: Mini 10-player list preview -->
        <div class="hidden lg:flex items-center space-x-2 pl-3 border-l border-slate-800/80">
          <div class="grid grid-cols-2 gap-x-2 gap-y-1">
            <div
              v-for="p in participants.slice(0, 10)"
              :key="p.participantId"
              class="flex items-center space-x-1"
              :title="`${p.gameName || '选手'} (${getChampionName(p.championId)})`"
            >
              <ChampionIcon
                :champion-id="p.championId"
                :size="16"
                round
                :class="p.puuid === participant.puuid ? 'ring-1 ring-[#0ac8b9]' : ''"
              />
              <span class="text-[10px] truncate max-w-14 text-slate-400 font-mono" :class="p.puuid === participant.puuid ? 'text-cyan-300 font-bold' : ''">
                {{ p.gameName }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Bottom Row: Items, Tags, Time ago -->
      <div class="flex items-center justify-between pt-2 mt-2 border-t border-slate-800/50">
        <!-- 6 Items + 1 Trinket slot -->
        <div class="flex items-center space-x-1">
          <ItemDisplay
            v-for="(item, idx) in (participant.items || []).slice(0, 6)"
            :key="idx"
            :item-id="item"
            :size="22"
          />
          <ItemDisplay
            :item-id="participant.items?.[6] || 0"
            :is-trinket="true"
            :size="22"
            class="ml-1"
          />
        </div>

        <!-- Badges & Tags -->
        <div class="flex items-center space-x-2">
          <ManyTags />
          <span class="text-[10px] text-slate-500 font-mono">
            {{ formatRelativeTime(basicInfo.gameCreation) }}
          </span>
        </div>
      </div>
    </div>

    <!-- Right Expand / Collapse Chevron Bar -->
    <div
      class="w-8 shrink-0 flex items-center justify-center border-l border-slate-800/80 bg-slate-900/40 group-hover:bg-slate-800/60 transition-colors"
      :title="isExpanded ? '收起详情' : '展开 7 个详情 Tab'"
    >
      <ChevronDown
        class="w-4 h-4 text-slate-400 group-hover:text-white transition-transform duration-300"
        :class="{ '-rotate-180': isExpanded }"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NPopover } from 'naive-ui'
import { ChevronDown } from 'lucide-vue-next'
import { useMatchCard } from './context'
import { MATCH_CARD_COLLAPSED_HEIGHT_PX } from './constants'
import { useCardBorderClass, useWinResultStyleType } from './utils/theme'
import { useGameResultName } from './utils/text'
import { formatDuration, formatRelativeTime } from './utils/time'
import { getChampionName, getQueueName } from './utils/resources'
import ChampionIcon from './widgets/ChampionIcon.vue'
import SummonerSpellDisplay from './widgets/SummonerSpellDisplay.vue'
import PerkDisplay from './widgets/PerkDisplay.vue'
import PerkstyleDisplay from './widgets/PerkstyleDisplay.vue'
import PositionIcon from './widgets/PositionIcon.vue'
import ItemDisplay from './widgets/ItemDisplay.vue'
import ManyTags from './widgets/ManyTags.vue'
import RadarChart from './widgets/RadarChart.vue'

defineEmits<{
  'toggle-expand': []
}>()

const { participant, team, teams, basicInfo, isExpanded, participants } = useMatchCard()

const winStyleType = computed(() => useWinResultStyleType(team.value?.winResult))
const cardBorderClass = computed(() => useCardBorderClass(team.value?.winResult))

const gameResultName = useGameResultName()
const gameResultText = computed(() =>
  gameResultName(team.value?.subteamPlacement, team.value?.winResult, team.value?.isSurrender)
)

const queueName = computed(() => getQueueName(basicInfo.value.queueId))

const damageSharePct = computed(() => {
  if (!participant.value || !team.value || !team.value.totalDamageDealtToChampions) return 0
  return Math.round(
    (participant.value.totalDamageDealtToChampions / team.value.totalDamageDealtToChampions) * 100
  )
})
</script>
