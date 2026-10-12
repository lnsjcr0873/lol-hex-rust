<template>
  <div class="h-full flex flex-col space-y-3">
    <!-- Minute Slider Header -->
    <div class="flex items-center justify-between p-3 bg-slate-900/90 rounded-xl border border-slate-800">
      <div class="flex items-center space-x-3">
        <span class="text-xs font-bold text-slate-200">逐帧时间线回放:</span>
        <span class="px-2 py-0.5 rounded bg-[#0f233a] text-[#0ac8b9] text-xs font-mono font-bold">
          第 {{ selectedMinute }} 分钟
        </span>
      </div>
      <div class="flex items-center space-x-3 w-72">
        <span class="text-[10px] text-slate-400 font-mono">00:00</span>
        <input
          type="range"
          min="0"
          :max="maxMinute"
          v-model.number="selectedMinute"
          class="flex-1 accent-[#0ac8b9] cursor-pointer"
        />
        <span class="text-[10px] text-slate-400 font-mono">{{ maxMinute }}:00</span>
      </div>
    </div>

    <!-- Dual pane: Left 10-player immediate stats table, Right map position replay -->
    <div class="flex-1 min-h-0 flex gap-4">
      <!-- Players instantaneous stats table -->
      <div class="flex-1 overflow-y-auto bg-slate-950/80 rounded-xl border border-slate-800 p-3">
        <table class="w-full text-xs text-left">
          <thead>
            <tr class="text-slate-400 border-b border-slate-800 pb-1">
              <th class="py-1">选手</th>
              <th class="py-1 text-center">等级</th>
              <th class="py-1 text-center">累计金币</th>
              <th class="py-1 text-center">补刀 (CS)</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-800/40 font-mono text-[11px]">
            <tr
              v-for="p in participants"
              :key="p.participantId"
              class="hover:bg-slate-900/40"
            >
              <td class="py-1.5 flex items-center space-x-2">
                <ChampionIcon :champion-id="p.championId" :size="22" round />
                <span class="font-sans font-semibold text-slate-200">{{ p.gameName }}</span>
              </td>
              <td class="py-1.5 text-center text-slate-300">
                Lv.{{ getInstantStat(p.participantId, 'level') }}
              </td>
              <td class="py-1.5 text-center text-amber-300 font-bold">
                {{ getInstantStat(p.participantId, 'gold') }}
              </td>
              <td class="py-1.5 text-center text-slate-300">
                {{ getInstantStat(p.participantId, 'cs') }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Minimap Replay -->
      <div class="w-64 bg-slate-950/80 rounded-xl border border-slate-800 p-2 flex flex-col items-center justify-center shrink-0">
        <div class="text-[11px] font-bold text-slate-300 mb-2">选手地图阵型分布</div>
        <MapPosition :map-id="basicInfo.mapId" :points="currentPositions" :size="220" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useMatchCard } from '../../context'
import ChampionIcon from '../../widgets/ChampionIcon.vue'
import MapPosition from '../../widgets/MapPosition.vue'

const { participants, frames, basicInfo } = useMatchCard()

const selectedMinute = ref(5)

const maxMinute = computed(() => {
  if (frames.value.length > 0) return frames.value.length - 1
  return Math.max(Math.floor(basicInfo.value.gameDuration / 60), 15)
})

const currentFrame = computed(() => {
  if (frames.value.length > 0) {
    return frames.value[selectedMinute.value] || frames.value[0]
  }
  return null
})

function getInstantStat(pid: number, type: 'level' | 'gold' | 'cs'): number {
  if (currentFrame.value?.participantFrames) {
    const pf = currentFrame.value.participantFrames[pid] || currentFrame.value.participantFrames[String(pid)]
    if (pf) {
      if (type === 'level') return pf.level || 1
      if (type === 'gold') return pf.totalGold || 500
      if (type === 'cs') return (pf.minionsKilled || 0) + (pf.jungleMinionsKilled || 0)
    }
  }

  // Graceful simulation
  const m = selectedMinute.value
  if (type === 'level') return Math.min(1 + Math.floor(m * 1.1), 18)
  if (type === 'gold') return 500 + m * 450
  if (type === 'cs') return m * 8
  return 0
}

const currentPositions = computed(() => {
  if (currentFrame.value?.participantFrames) {
    const pts: Array<{ x: number; y: number }> = []
    for (const pid in currentFrame.value.participantFrames) {
      const pos = currentFrame.value.participantFrames[pid]?.position
      if (pos && typeof pos.x === 'number') {
        pts.push({ x: pos.x, y: pos.y })
      }
    }
    if (pts.length > 0) return pts
  }

  // Fallback demo positions
  return participants.value.map((_, i) => ({
    x: 2000 + i * 1100,
    y: 2000 + (i % 3) * 4000
  }))
})
</script>
