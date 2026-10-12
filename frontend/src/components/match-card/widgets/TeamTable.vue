<template>
  <div v-if="team" class="overflow-hidden rounded-xl border border-solid transition-colors" :class="tableTheme.border">
    <!-- Team Header -->
    <div class="px-4 py-2 flex items-center justify-between text-xs" :class="tableTheme.header">
      <!-- Left: Result, Team Name, KDA, Gold, Objectives -->
      <div class="flex items-center space-x-3.5 flex-wrap">
        <span
          class="font-black px-2 py-0.5 rounded text-[11px] uppercase tracking-wider"
          :class="team.winResult === 'win' ? 'bg-blue-950/90 text-blue-300 border border-blue-500/50' : 'bg-red-950/90 text-red-300 border border-red-500/50'"
        >
          {{ formatResultName(team.subteamPlacement, team.winResult, team.isSurrender) }}
        </span>

        <span class="font-bold text-slate-100 text-xs">{{ teamDisplayName }}</span>

        <span class="font-mono font-bold text-slate-200">
          {{ team.totalKills }} / {{ team.totalDeaths }} / {{ team.totalAssists }}
        </span>

        <span class="font-mono font-bold text-amber-300">
          {{ (team.totalGoldEarned / 1000).toFixed(1) }}k 金
        </span>

        <!-- Objective Icons -->
        <div v-if="team.teamInfo" class="flex items-center space-x-2 text-slate-400 pl-2 border-l border-slate-700/60">
          <div class="flex items-center space-x-1" title="防御塔">
            <Tower class="w-3.5 h-3.5 text-amber-400" />
            <span class="font-mono">{{ team.teamInfo.objectives.tower.kills }}</span>
          </div>
          <div class="flex items-center space-x-1" title="水晶">
            <Inhibitor class="w-3.5 h-3.5 text-purple-400" />
            <span class="font-mono">{{ team.teamInfo.objectives.inhibitor.kills }}</span>
          </div>
          <div class="flex items-center space-x-1" title="巨龙">
            <Dragon class="w-3.5 h-3.5 text-orange-400" />
            <span class="font-mono">{{ team.teamInfo.objectives.dragon.kills }}</span>
          </div>
          <div class="flex items-center space-x-1" title="男爵">
            <Baron class="w-3.5 h-3.5 text-rose-400" />
            <span class="font-mono">{{ team.teamInfo.objectives.baron.kills }}</span>
          </div>
          <div v-if="team.teamInfo.objectives.horde" class="flex items-center space-x-1" title="虚空巢虫">
            <VoidGrub class="w-3.5 h-3.5 text-indigo-400" />
            <span class="font-mono">{{ team.teamInfo.objectives.horde.kills }}</span>
          </div>
          <div v-if="team.teamInfo.objectives.riftHerald" class="flex items-center space-x-1" title="峡谷先锋">
            <RiftHerald class="w-3.5 h-3.5 text-sky-400" />
            <span class="font-mono">{{ team.teamInfo.objectives.riftHerald.kills }}</span>
          </div>
          <div v-if="team.teamInfo.objectives.atakhan && team.teamInfo.objectives.atakhan.kills > 0" class="flex items-center space-x-1" title="阿塔坎">
            <Atakhan class="w-3.5 h-3.5 text-red-400" />
            <span class="font-mono">{{ team.teamInfo.objectives.atakhan.kills }}</span>
          </div>
        </div>
      </div>

      <!-- Right: Bans -->
      <div v-if="team.teamInfo && team.teamInfo.bans.length > 0" class="flex items-center space-x-1.5">
        <span class="text-[10px] text-slate-400">禁用:</span>
        <div class="flex items-center space-x-1">
          <ChampionIcon
            v-for="ban in team.teamInfo.bans.slice(0, 5)"
            :key="ban.championId"
            :champion-id="ban.championId"
            :size="18"
            class="rounded-sm opacity-80 hover:opacity-100"
          />
        </div>
      </div>
    </div>

    <!-- Table Column Headers -->
    <div class="px-3 py-1 bg-slate-900/90 border-b border-slate-800 text-[10px] text-slate-400 flex items-center justify-between select-none">
      <div class="w-56">召唤师 / 英雄 / 位置</div>
      <div class="w-20 text-center">战损 (KDA)</div>
      <div class="w-24 text-center">伤害输出</div>
      <div class="w-24 text-center">伤害承受</div>
      <div class="w-20 text-right">补刀 / 经济</div>
      <div class="w-48 pl-3">出装构建 (6槽 + 饰品)</div>
      <div class="w-16 text-right pr-2">视野得分</div>
    </div>

    <!-- Participants Rows -->
    <div class="divide-y divide-slate-800/60 bg-slate-950/80">
      <div
        v-for="p in teamParticipants"
        :key="p.participantId"
        class="px-3 py-2 flex items-center justify-between hover:bg-slate-900/60 transition-colors"
        :class="{ 'bg-cyan-950/20 ring-1 ring-cyan-500/20': p.puuid === focusedPuuid }"
      >
        <!-- Hero, Spells, Name & Position -->
        <div class="flex items-center space-x-2.5 w-56 shrink-0">
          <div class="relative shrink-0">
            <ChampionIcon :champion-id="p.championId" :size="36" round />
            <span class="absolute -bottom-1 -right-1 text-[8px] bg-black/90 px-1 rounded text-slate-300 font-bold font-mono">
              {{ p.championLevel }}
            </span>
          </div>

          <!-- Spells & Keystone -->
          <div class="flex flex-col space-y-0.5 shrink-0">
            <div class="flex space-x-0.5">
              <SummonerSpellDisplay :spell-id="p.spells[0]" :size="15" />
              <SummonerSpellDisplay :spell-id="p.spells[1]" :size="15" />
            </div>
            <div class="flex space-x-0.5" v-if="p.perks">
              <PerkDisplay :perk-id="p.perks.styles?.[0]?.selections?.[0]?.perk" :size="15" />
              <PerkstyleDisplay :perkstyle-id="p.perks.styles?.[1]?.style" :size="15" />
            </div>
          </div>

          <!-- Summoner Info -->
          <div class="min-w-0 flex flex-col">
            <div class="text-xs font-bold text-slate-100 truncate flex items-center space-x-1">
              <span class="truncate">{{ p.gameName || '召唤师' }}</span>
              <span v-if="p.tagLine" class="text-[9px] text-slate-500 font-mono">#{{ p.tagLine }}</span>
            </div>
            <div class="text-[10px] text-slate-400 flex items-center space-x-1 mt-0.5">
              <PositionIcon :position="p.position" :size="11" class="text-slate-400" />
              <span>{{ formatParticipantRole(p.position || '') }}</span>
              <span v-if="p.tier">· <span class="text-amber-300/90 font-mono">{{ p.tier }}</span></span>
            </div>
          </div>
        </div>

        <!-- KDA -->
        <div class="w-20 text-center shrink-0">
          <div class="text-xs font-bold font-mono text-slate-100">
            {{ p.kills }}/<span class="text-red-400">{{ p.deaths }}</span>/{{ p.assists }}
          </div>
          <div class="text-[10px] font-mono text-slate-400">
            <span class="text-slate-300 font-semibold">{{ p.kda.toFixed(2) }}</span>
            <span> ({{ (p.killParticipation * 100).toFixed(0) }}%)</span>
          </div>
        </div>

        <!-- Damage Dealt Bar -->
        <div class="w-24 text-center shrink-0">
          <DamageBarWithPopover
            title="对英雄伤害"
            :physical-damage="p.physicalDamageDealtToChampions"
            :magic-damage="p.magicDamageDealtToChampions"
            :true-damage="p.trueDamageDealtToChampions"
            :total-damage="p.totalDamageDealtToChampions"
            :baseline-damage="team.maxDamageDealtToChampions || 1"
            :width="64"
            :height="5"
          />
        </div>

        <!-- Damage Taken Bar -->
        <div class="w-24 text-center shrink-0">
          <DamageBarWithPopover
            title="承受伤害"
            :physical-damage="p.physicalDamageTaken"
            :magic-damage="p.magicDamageTaken"
            :true-damage="p.trueDamageTaken"
            :total-damage="p.totalDamageTaken"
            :baseline-damage="team.maxDamageTaken || 1"
            :width="64"
            :height="5"
          />
        </div>

        <!-- CS & Gold -->
        <div class="w-20 text-right shrink-0">
          <div class="text-xs font-mono font-bold text-amber-300">
            {{ (p.goldEarned / 1000).toFixed(1) }}k 金
          </div>
          <div class="text-[10px] font-mono text-slate-400">
            {{ p.cs }} 刀 ({{ ((p.cs / Math.max(basicInfo.gameDuration / 60, 1))).toFixed(1) }}/分)
          </div>
        </div>

        <!-- Items 6 slots + 1 trinket -->
        <div class="w-48 pl-3 flex items-center space-x-1 shrink-0">
          <ItemDisplay
            v-for="(item, idx) in (p.items || []).slice(0, 6)"
            :key="idx"
            :item-id="item"
            :size="20"
          />
          <ItemDisplay
            :item-id="p.items?.[6] || 0"
            :is-trinket="true"
            :size="20"
            class="ml-1"
          />
        </div>

        <!-- Vision -->
        <div class="w-16 text-right pr-2 shrink-0 font-mono text-xs">
          <div class="font-bold text-slate-200">{{ p.visionScore }}</div>
          <div class="text-[9px] text-slate-500">
            {{ p.wardsPlaced }}/{{ p.wardsKilled }}/{{ p.detectorWardsPlaced }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useMatchCard } from '../context'
import { useGameResultName, useTeamName, formatParticipantRole } from '../utils/text'
import ChampionIcon from './ChampionIcon.vue'
import ItemDisplay from './ItemDisplay.vue'
import PerkDisplay from './PerkDisplay.vue'
import PerkstyleDisplay from './PerkstyleDisplay.vue'
import SummonerSpellDisplay from './SummonerSpellDisplay.vue'
import PositionIcon from './PositionIcon.vue'
import DamageBarWithPopover from './DamageBarWithPopover.vue'
import Tower from '../icons/Tower.vue'
import Inhibitor from '../icons/Inhibitor.vue'
import Dragon from '../icons/Dragon.vue'
import Baron from '../icons/Baron.vue'
import VoidGrub from '../icons/VoidGrub.vue'
import RiftHerald from '../icons/RiftHerald.vue'
import Atakhan from '../icons/Atakhan.vue'

const props = defineProps<{
  teamIdentifier: string
}>()

const { teams, participants, basicInfo, puuid: focusedPuuid } = useMatchCard()

const team = computed(() => {
  return teams.value.teamStatMap?.[props.teamIdentifier]
})

const teamParticipants = computed(() => {
  return participants.value.filter((p) => p.teamIdentifier === props.teamIdentifier)
})

const formatResultName = useGameResultName()
const getTeamName = useTeamName()

const teamDisplayName = computed(() => {
  return getTeamName(props.teamIdentifier)
})

const tableTheme = computed(() => {
  if (team.value?.winResult === 'win') {
    return {
      border: 'border-blue-900/60 shadow-[0_0_15px_rgba(59,130,246,0.06)]',
      header: 'bg-blue-950/60 border-b border-blue-900/60'
    }
  }
  return {
    border: 'border-red-950/60 shadow-[0_0_15px_rgba(239,68,68,0.06)]',
    header: 'bg-red-950/60 border-b border-red-900/60'
  }
})
</script>
