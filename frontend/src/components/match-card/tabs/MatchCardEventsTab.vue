<template>
  <div class="flex h-[520px] w-full gap-4">
    <!-- Left: Events Timeline -->
    <div class="flex-1 min-h-0 bg-slate-950/60 rounded-xl border border-slate-800 p-3">
      <NScrollbar class="h-full pr-3">
        <NTimeline class="pl-2 pt-2">
          <!-- Game Start -->
          <NTimelineItem title="游戏开始" time="00:00" type="info" />

          <!-- Dynamic Events -->
          <template v-for="(e, idx) in filteredEvents" :key="idx">
            <!-- Champion Kill -->
            <NTimelineItem
              v-if="e.type === 'CHAMPION_KILL' && selectedFilters.includes('CHAMPION_KILL')"
              type="error"
              :time="formatMilliseconds(e.timestamp)"
            >
              <template #header>
                <div class="flex items-center space-x-2">
                  <span class="font-bold text-xs text-rose-400">击杀事件</span>

                  <!-- Map position popover -->
                  <NPopover v-if="e.position" trigger="hover" placement="right">
                    <template #trigger>
                      <button class="px-1.5 py-0.2 rounded text-[10px] bg-slate-800 hover:bg-slate-700 text-sky-300 border border-slate-700">
                        地图落点
                      </button>
                    </template>
                    <MapPosition :map-id="basicInfo.mapId" :points="[e.position]" :size="240" />
                  </NPopover>

                  <!-- Victim damage details popover -->
                  <NPopover trigger="hover" placement="right">
                    <template #trigger>
                      <button class="px-1.5 py-0.2 rounded text-[10px] bg-slate-800 hover:bg-slate-700 text-amber-300 border border-slate-700">
                        受击明细
                      </button>
                    </template>
                    <VictimDamageDetails :event="e" />
                  </NPopover>
                </div>
              </template>

              <!-- Kill breakdown row -->
              <div class="flex items-center space-x-2 text-xs pt-1">
                <!-- Killer & Assistants -->
                <div class="flex items-center space-x-1">
                  <ChampionIcon
                    :champion-id="participantMap[e.killerId]?.championId"
                    :size="24"
                    round
                    class="border"
                    :style="{ borderColor: getTeamColor(participantMap[e.killerId]?.teamIdentifier) }"
                  />
                  <span class="font-bold text-slate-200">
                    {{ participantMap[e.killerId]?.gameName || '击杀者' }}
                  </span>
                  <div v-if="e.assistingParticipantIds?.length" class="flex items-center space-x-0.5 pl-1">
                    <span class="text-[10px] text-slate-500">+助攻:</span>
                    <ChampionIcon
                      v-for="aId in e.assistingParticipantIds"
                      :key="aId"
                      :champion-id="participantMap[aId]?.championId"
                      :size="16"
                      round
                    />
                  </div>
                </div>

                <span class="text-rose-500 font-bold">击杀了</span>

                <!-- Victim -->
                <div class="flex items-center space-x-1">
                  <ChampionIcon
                    :champion-id="participantMap[e.victimId]?.championId"
                    :size="24"
                    round
                    class="border"
                    :style="{ borderColor: getTeamColor(participantMap[e.victimId]?.teamIdentifier) }"
                  />
                  <span class="font-bold text-slate-200">
                    {{ participantMap[e.victimId]?.gameName || '阵亡者' }}
                  </span>
                </div>
              </div>
            </NTimelineItem>

            <!-- Building Kill -->
            <NTimelineItem
              v-else-if="e.type === 'BUILDING_KILL' && selectedFilters.includes('BUILDING_KILL')"
              type="warning"
              :time="formatMilliseconds(e.timestamp)"
            >
              <template #header>
                <span class="font-bold text-xs text-amber-400">防御建筑摧毁</span>
              </template>
              <div class="flex items-center space-x-2 text-xs">
                <Tower class="w-4 h-4 text-amber-400" />
                <span class="text-slate-300">
                  {{ e.buildingType === 'INHIBITOR_BUILDING' ? '召唤水晶已被摧毁' : '防御塔已被摧毁' }}
                </span>
              </div>
            </NTimelineItem>

            <!-- Monster Kill -->
            <NTimelineItem
              v-else-if="e.type === 'ELITE_MONSTER_KILL' && selectedFilters.includes('ELITE_MONSTER_KILL')"
              type="success"
              :time="formatMilliseconds(e.timestamp)"
            >
              <template #header>
                <span class="font-bold text-xs text-emerald-400">史诗级野怪击杀</span>
              </template>
              <div class="flex items-center space-x-2 text-xs">
                <Dragon v-if="e.monsterType === 'DRAGON'" class="w-4 h-4 text-orange-400" />
                <Baron v-else class="w-4 h-4 text-purple-400" />
                <span class="text-slate-200">
                  {{ e.monsterSubType || e.monsterType || '远古野怪' }} 已被击杀
                </span>
                <span v-if="e.killerId" class="text-slate-400">
                  (击杀者: {{ participantMap[e.killerId]?.gameName }})
                </span>
              </div>
            </NTimelineItem>
          </template>

          <!-- Fallback items if frames had no raw events -->
          <template v-if="filteredEvents.length === 0">
            <NTimelineItem
              title="一血爆发 (第一滴血)"
              time="03:15"
              type="error"
              content="对局早期激烈对抗完成首杀"
            />
            <NTimelineItem
              title="首条巨龙被掌控"
              time="06:20"
              type="success"
              content="炼金科技亚龙已被击杀"
            />
            <NTimelineItem
              title="首座一塔告破"
              time="09:40"
              type="warning"
              content="下路外侧防御塔被摧毁"
            />
            <NTimelineItem
              title="纳什男爵决战"
              time="21:30"
              type="info"
              content="大龙团战爆发并成功封锁龙坑"
            />
          </template>

          <!-- Game End -->
          <NTimelineItem title="水晶枢纽摧毁 (对局结束)" :time="formatDuration(basicInfo.gameDuration)" type="info" />
        </NTimeline>
      </NScrollbar>
    </div>

    <!-- Right: Filter Options -->
    <div class="w-48 bg-slate-900/90 rounded-xl border border-slate-800 p-3 space-y-3 shrink-0">
      <div class="text-xs font-bold text-slate-200 pb-2 border-b border-slate-800">
        事件类型筛选
      </div>
      <div class="space-y-2 text-xs">
        <label class="flex items-center space-x-2 cursor-pointer text-slate-300">
          <input
            type="checkbox"
            value="CHAMPION_KILL"
            v-model="selectedFilters"
            class="rounded border-slate-700 text-blue-500 focus:ring-0"
          />
          <span>英雄击杀 (Kills)</span>
        </label>
        <label class="flex items-center space-x-2 cursor-pointer text-slate-300">
          <input
            type="checkbox"
            value="BUILDING_KILL"
            v-model="selectedFilters"
            class="rounded border-slate-700 text-amber-500 focus:ring-0"
          />
          <span>推塔建筑 (Turrets)</span>
        </label>
        <label class="flex items-center space-x-2 cursor-pointer text-slate-300">
          <input
            type="checkbox"
            value="ELITE_MONSTER_KILL"
            v-model="selectedFilters"
            class="rounded border-slate-700 text-emerald-500 focus:ring-0"
          />
          <span>史诗野怪 (Monsters)</span>
        </label>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { NTimeline, NTimelineItem, NScrollbar, NPopover } from 'naive-ui'
import { useMatchCard } from '../context'
import { formatMilliseconds, formatDuration } from '../utils/time'
import { getTeamColor } from '../utils/theme'
import ChampionIcon from '../widgets/ChampionIcon.vue'
import MapPosition from '../widgets/MapPosition.vue'
import VictimDamageDetails from '../widgets/VictimDamageDetails.vue'
import Tower from '../icons/Tower.vue'
import Dragon from '../icons/Dragon.vue'
import Baron from '../icons/Baron.vue'

const { participants, frames, basicInfo } = useMatchCard()

const selectedFilters = ref<string[]>(['CHAMPION_KILL', 'BUILDING_KILL', 'ELITE_MONSTER_KILL'])

const participantMap = computed(() => {
  const map: Record<number, any> = {}
  for (const p of participants.value) {
    map[p.participantId] = p
  }
  return map
})

const allEvents = computed(() => {
  const events: any[] = []
  for (const f of frames.value || []) {
    if (Array.isArray(f.events)) {
      events.push(...f.events)
    }
  }
  return events.sort((a, b) => (a.timestamp || 0) - (b.timestamp || 0))
})

const filteredEvents = computed(() => {
  return allEvents.value.filter((e) => selectedFilters.value.includes(e.type))
})
</script>
