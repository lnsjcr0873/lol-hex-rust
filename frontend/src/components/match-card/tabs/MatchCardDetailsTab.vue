<template>
  <div class="space-y-3">
    <!-- Top Metadata Bar -->
    <div class="px-3 py-1.5 bg-slate-900/90 rounded-lg border border-slate-800 text-[11px] text-slate-400 flex items-center justify-between select-none">
      <div class="flex items-center space-x-3">
        <span>对局 ID: <strong class="text-slate-200 font-mono">{{ basicInfo?.gameId || 0 }}</strong> ({{ (basicInfo?.dataSource || 'lcu').toUpperCase() }})</span>
        <span>版本: <span class="font-mono text-slate-300">{{ basicInfo?.gameVersion || '14.x' }}</span></span>
        <span>模式: <span class="text-[#0ac8b9]">{{ basicInfo?.gameMode || 'MATCH' }}</span></span>
      </div>
      <div>
        <span>时长: <strong class="text-slate-200 font-mono">{{ formatDuration(basicInfo?.gameDuration || 0) }}</strong></span>
      </div>
    </div>

    <!-- Search filter bar -->
    <div class="flex items-center justify-between">
      <div class="w-64">
        <NInput
          v-model:value="filterText"
          size="small"
          placeholder="搜索局内指标 (如: 暴击, 伤害, 控制, 金币)"
          clearable
        >
          <template #prefix>
            <Search class="w-3.5 h-3.5 text-slate-400" />
          </template>
        </NInput>
      </div>
      <div class="text-xs text-slate-400">
        共匹配 {{ filteredGroups.reduce((acc, g) => acc + g.items.length, 0) }} 项原生指标 (悬浮指标名可查看10人横向对比图)
      </div>
    </div>

    <!-- 10-Player Matrix Comparison Table -->
    <NScrollbar x-scrollable class="rounded-xl border border-slate-800 bg-slate-950/80">
      <table class="w-full text-xs text-left border-collapse select-none">
        <!-- Table Header (10 Players) -->
        <thead>
          <tr class="bg-slate-900/95 border-b border-slate-800 text-slate-300">
            <th class="p-2.5 w-44 min-w-44 sticky left-0 z-20 bg-slate-900/95 font-bold border-r border-slate-800">
              数据指标分类
            </th>
            <th
              v-for="p in rawStats"
              :key="p.participantId"
              class="p-2 text-center min-w-24"
            >
              <div class="flex flex-col items-center space-y-1">
                <ChampionIcon
                  :champion-id="p.championId"
                  :size="26"
                  round
                  class="border-2"
                  :style="{ borderColor: getTeamColor(p.identity?.teamIdentifier) }"
                />
                <span class="text-[10px] truncate max-w-20 font-semibold text-slate-200">
                  {{ p.identity?.gameName || `P${p.participantId}` }}
                </span>
              </div>
            </th>
          </tr>
        </thead>

        <!-- Table Body categorized by 18 groups -->
        <tbody class="divide-y divide-slate-800/40 font-mono text-[11px]">
          <template v-for="g in filteredGroups" :key="g.group">
            <!-- Group Header -->
            <tr class="bg-slate-900/60 font-sans font-bold text-xs text-cyan-400 border-t-2 border-slate-800">
              <td class="p-2 sticky left-0 z-10 bg-slate-900/95 pl-3 border-r border-slate-800">
                📁 {{ g.name || g.group }}
              </td>
              <td :colspan="rawStats.length"></td>
            </tr>

            <!-- Metric Rows -->
            <tr
              v-for="item in g.items"
              :key="item.key"
              class="hover:bg-slate-900/40 transition-colors"
            >
              <!-- Metric Label (with Popover bar chart) -->
              <NPopover trigger="hover" placement="right">
                <template #trigger>
                  <td class="p-2 sticky left-0 z-10 bg-slate-950/95 font-sans font-semibold text-slate-300 hover:text-cyan-300 cursor-help border-r border-slate-800 truncate">
                    {{ getStatKeyTranslation(item.key) }}
                  </td>
                </template>
                <StatsBarChart
                  :title="getStatKeyTranslation(item.key)"
                  :chart-data="getMetricComparison(item.key)"
                />
              </NPopover>

              <!-- 10 Player Values -->
              <td
                v-for="p in rawStats"
                :key="p.participantId"
                class="p-2 text-center text-slate-300"
              >
                {{ formatCellValue(p[item.key], item.render) }}
              </td>
            </tr>
          </template>
        </tbody>
      </table>
    </NScrollbar>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { NInput, NScrollbar, NPopover } from 'naive-ui'
import { Search } from 'lucide-vue-next'
import { useMatchCard } from '../context'
import { useRawDetails } from '../utils/details-table/raw-details'
import { RENDER_GROUPS } from '../utils/details-table/groups'
import { getStatKeyTranslation } from '../utils/text'
import { formatDuration } from '../utils/time'
import { getTeamColor } from '../utils/theme'
import ChampionIcon from '../widgets/ChampionIcon.vue'
import StatsBarChart from '../widgets/StatsBarChart.vue'

const { basicInfo } = useMatchCard()
const rawStats = useRawDetails()
const filterText = ref('')

const filteredGroups = computed(() => {
  const query = filterText.value.trim().toLowerCase()
  if (!query) return RENDER_GROUPS

  return RENDER_GROUPS.map((g) => {
    const items = g.items.filter((item) => {
      const cnName = getStatKeyTranslation(item.key).toLowerCase()
      return cnName.includes(query) || item.key.toLowerCase().includes(query)
    })
    return { ...g, items }
  }).filter((g) => g.items.length > 0)
})

function formatCellValue(val: any, renderType?: any): string {
  if (val === undefined || val === null) return '-'
  if (typeof renderType === 'function') {
    return String(renderType(val))
  }
  if (renderType === 'boolean') {
    return val ? '✔' : '✖'
  }
  if (renderType === 'akari-score') {
    const num = (val && typeof val === 'object' && val.total !== undefined) ? Number(val.total) : Number(val)
    return Number.isFinite(num) ? num.toFixed(2) : '0.00'
  }
  if (renderType === 'percentage') {
    const num = Number(val)
    return Number.isFinite(num) ? `${num.toFixed(1)}%` : '0.0%'
  }
  if (renderType === 'compat') {
    const num = Number(val)
    if (Number.isFinite(num)) {
      return num >= 1000 ? `${(num / 1000).toFixed(1)}k` : num.toString()
    }
    return String(val)
  }
  if (renderType === 'integer') {
    const num = Number(val)
    return Number.isFinite(num) ? Math.round(num).toLocaleString() : '-'
  }
  if (typeof val === 'number') {
    return Number.isFinite(val) ? val.toLocaleString() : '-'
  }
  return String(val)
}

function getMetricComparison(key: string) {
  return rawStats.value.map((p: any) => {
    const raw = typeof p[key] === 'number' ? p[key] : (p[key]?.total ?? 0)
    const val = Number(raw)
    return {
      participantId: p.participantId,
      value: Number.isFinite(val) ? val : 0
    }
  })
}
</script>
