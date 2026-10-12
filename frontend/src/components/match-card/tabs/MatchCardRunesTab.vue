<template>
  <div class="relative flex">
    <!-- Main Runes Scroll Container -->
    <NScrollbar class="max-h-[560px] flex-1 pr-16">
      <div class="space-y-4">
        <div
          v-for="p in participants"
          :key="p.participantId"
          :id="`runes-p-${p.participantId}`"
          class="p-4 rounded-xl bg-slate-900/60 border border-slate-800"
        >
          <!-- Player Header -->
          <div class="flex items-center justify-between pb-3 border-b border-slate-800/80 mb-3">
            <div class="flex items-center space-x-2.5">
              <ChampionIcon
                :champion-id="p.championId"
                :size="32"
                round
                class="border-2"
                :style="{ borderColor: getTeamColor(p.teamIdentifier) }"
              />
              <div>
                <div class="text-xs font-bold text-slate-100 flex items-center space-x-1.5">
                  <span>{{ p.gameName || '召唤师' }}</span>
                  <span v-if="p.tagLine" class="text-[10px] text-slate-500 font-mono">#{{ p.tagLine }}</span>
                </div>
                <div class="text-[10px] text-slate-400 mt-0.5 flex items-center space-x-1">
                  <span>{{ formatParticipantRole(p.position || '') }}</span>
                  <span>·</span>
                  <span class="text-amber-300 font-mono">{{ p.tier || 'UNRANKED' }}</span>
                </div>
              </div>
            </div>

            <!-- Stat Perks Shards -->
            <div v-if="p.perks?.statPerks" class="flex items-center space-x-2 px-2.5 py-1 rounded-lg bg-slate-950 border border-slate-800">
              <span class="text-[10px] text-slate-400">属性碎片:</span>
              <div class="flex items-center space-x-1">
                <PerkDisplay :perk-id="p.perks.statPerks.offense" :size="16" />
                <PerkDisplay :perk-id="p.perks.statPerks.flex" :size="16" />
                <PerkDisplay :perk-id="p.perks.statPerks.defense" :size="16" />
              </div>
            </div>
          </div>

          <!-- Primary & Secondary Styles -->
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <!-- Primary Style -->
            <div v-if="p.perks?.styles?.[0]" class="p-3 rounded-lg bg-slate-950/70 border border-slate-800/80">
              <div class="flex items-center space-x-2 pb-2 mb-2 border-b border-slate-800/60">
                <PerkstyleDisplay :perkstyle-id="p.perks.styles[0].style" :size="20" />
                <span class="text-xs font-bold text-slate-200">
                  主系: {{ getPerkName(p.perks.styles[0].style) }}
                </span>
              </div>
              <div class="space-y-2">
                <div
                  v-for="(sel, sIdx) in p.perks.styles[0].selections"
                  :key="sIdx"
                  class="flex items-start space-x-2.5 text-xs p-1 rounded hover:bg-slate-900/40"
                >
                  <PerkDisplay :perk-id="sel.perk" :size="22" class="mt-0.5 shrink-0" />
                  <div class="min-w-0">
                    <div class="font-bold text-slate-200">{{ getPerkName(sel.perk) }}</div>
                    <div class="text-[11px] text-[#0ac8b9] font-mono mt-0.5">
                      局内收益:
                      <span v-if="sel.var1 !== undefined">数值1: {{ sel.var1.toLocaleString() }} </span>
                      <span v-if="sel.var2 && sel.var2 > 0">| 数值2: {{ sel.var2.toLocaleString() }} </span>
                      <span v-if="sel.var3 && sel.var3 > 0">| 数值3: {{ sel.var3.toLocaleString() }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Secondary Style -->
            <div v-if="p.perks?.styles?.[1]" class="p-3 rounded-lg bg-slate-950/70 border border-slate-800/80">
              <div class="flex items-center space-x-2 pb-2 mb-2 border-b border-slate-800/60">
                <PerkstyleDisplay :perkstyle-id="p.perks.styles[1].style" :size="20" />
                <span class="text-xs font-bold text-slate-200">
                  副系: {{ getPerkName(p.perks.styles[1].style) }}
                </span>
              </div>
              <div class="space-y-2">
                <div
                  v-for="(sel, sIdx) in p.perks.styles[1].selections"
                  :key="sIdx"
                  class="flex items-start space-x-2.5 text-xs p-1 rounded hover:bg-slate-900/40"
                >
                  <PerkDisplay :perk-id="sel.perk" :size="22" class="mt-0.5 shrink-0" />
                  <div class="min-w-0">
                    <div class="font-bold text-slate-200">{{ getPerkName(sel.perk) }}</div>
                    <div class="text-[11px] text-[#0ac8b9] font-mono mt-0.5" v-if="sel.var1 || sel.var2 || sel.var3">
                      局内收益: {{ sel.var1 ? sel.var1.toLocaleString() : 0 }}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </NScrollbar>

    <!-- Right Floating Player Navigator Bar -->
    <div class="w-12 shrink-0 flex flex-col items-center py-2 space-y-2 bg-slate-900/80 border border-slate-800 rounded-xl ml-2 shadow-lg h-fit sticky top-2">
      <span class="text-[9px] text-slate-400 font-bold mb-1">选手</span>
      <div
        v-for="p in participants"
        :key="p.participantId"
        @click="scrollToParticipant(p.participantId)"
        class="cursor-pointer transition-transform hover:scale-110 relative"
        :title="`${p.gameName || '选手'} (${getChampionName(p.championId)})`"
      >
        <ChampionIcon
          :champion-id="p.championId"
          :size="26"
          round
          class="border"
          :style="{ borderColor: getTeamColor(p.teamIdentifier) }"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NScrollbar } from 'naive-ui'
import { useMatchCard } from '../context'
import { getChampionName, getPerkName } from '../utils/resources'
import { formatParticipantRole } from '../utils/text'
import { getTeamColor } from '../utils/theme'
import ChampionIcon from '../widgets/ChampionIcon.vue'
import PerkDisplay from '../widgets/PerkDisplay.vue'
import PerkstyleDisplay from '../widgets/PerkstyleDisplay.vue'

const { participants } = useMatchCard()

function scrollToParticipant(pid: number) {
  const el = document.getElementById(`runes-p-${pid}`)
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'center' })
  }
}
</script>
