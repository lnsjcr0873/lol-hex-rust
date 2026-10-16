<template>
  <div class="h-screen w-screen flex flex-col bg-[#080d1a]/95 border border-cyan-500/30 rounded-xl overflow-hidden backdrop-blur-2xl shadow-2xl select-none text-slate-200">
    <!-- Floating Window Drag Titlebar -->
    <div
      data-tauri-drag-region
      class="h-10 px-4 bg-[#050811]/90 border-b border-slate-800/90 flex items-center justify-between shrink-0 cursor-move"
    >
      <!-- Left: Logo & Phase Status -->
      <div class="flex items-center space-x-2.5 pointer-events-none">
        <span class="w-2.5 h-2.5 rounded-full" :class="statusColorClass"></span>
        <span class="text-xs font-bold text-slate-100 tracking-wide flex items-center space-x-1.5">
          <span>HexBrain 对局悬浮窗</span>
          <span class="text-[10px] text-cyan-400 font-mono px-1.5 py-0.2 rounded bg-cyan-950/70 border border-cyan-500/30">
            FLOAT
          </span>
        </span>
        <span class="text-[11px] text-slate-400">
          {{ phaseLabel }}
        </span>
      </div>

      <!-- Right: Controls -->
      <div class="flex items-center space-x-1">
        <!-- Always On Top Pin Toggle -->
        <button
          @click="toggleAlwaysOnTop"
          class="p-1.5 rounded-lg hover:bg-slate-800 text-xs transition"
          :class="isAlwaysOnTop ? 'text-cyan-400 bg-cyan-950/50' : 'text-slate-400 hover:text-slate-200'"
          :title="isAlwaysOnTop ? '已置顶 (点击取消置顶)' : '点击置顶悬浮窗'"
        >
          <Pin class="w-3.5 h-3.5" :class="{ 'rotate-45': !isAlwaysOnTop }" />
        </button>

        <!-- Demo Mode Quick Toggle -->
        <button
          @click="toggleDemoMode"
          class="px-2 py-0.5 rounded-lg text-[11px] font-semibold transition border"
          :class="isDemoMode ? 'bg-amber-950/80 text-amber-300 border-amber-600/50' : 'bg-slate-900 text-slate-400 hover:text-slate-200 border-slate-800'"
          title="切换模拟演练数据与实时监听"
        >
          {{ isDemoMode ? '演练中' : '演练' }}
        </button>

        <!-- Refresh Button -->
        <button
          @click="() => loadMatchData(true)"
          class="p-1.5 rounded-lg hover:bg-slate-800 text-slate-400 hover:text-slate-200 text-xs transition"
          title="刷新对局数据"
        >
          <RotateCw class="w-3.5 h-3.5" :class="{ 'animate-spin': isLoading }" />
        </button>

        <!-- Minimize / Hide Button -->
        <button
          @click="handleClose"
          class="p-1.5 rounded-lg hover:bg-slate-800 text-slate-400 hover:text-rose-400 text-xs transition"
          title="隐藏悬浮窗"
        >
          <X class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Main Viewport -->
    <div class="flex-1 flex flex-col overflow-hidden relative">
      <!-- Waiting / Offline State -->
      <div
        v-if="!isDemoMode && (!matchData?.lcu_connected || !matchData?.is_active)"
        class="flex-1 flex flex-col items-center justify-center p-6 text-center space-y-3"
      >
        <div class="w-12 h-12 rounded-xl bg-slate-900 border border-slate-800 flex items-center justify-center text-cyan-400 shadow-inner">
          <Radio v-if="matchData?.lcu_connected" class="w-6 h-6 animate-pulse text-emerald-400" />
          <AlertCircle v-else class="w-6 h-6 text-amber-400" />
        </div>
        <div>
          <h3 class="text-sm font-bold text-slate-200">
            {{ matchData?.lcu_connected ? '客户端已就绪 · 等待进入选人或加载对局' : '未检测到英雄联盟客户端' }}
          </h3>
          <p class="text-[11px] text-slate-400 max-w-sm mt-1 leading-relaxed">
            进入对局后，悬浮窗将自动穿透展示双方 10 人近期胜率、分路对位与车队开黑标记。
          </p>
        </div>
        <div class="flex items-center space-x-2 pt-2">
          <button
            @click="toggleDemoMode"
            class="px-3 py-1.5 rounded-xl bg-cyan-950/80 hover:bg-cyan-900/80 border border-cyan-500/50 text-cyan-300 text-xs font-semibold shadow-sm transition active:scale-95"
          >
            开启演练演示对局
          </button>
          <button
            @click="() => loadMatchData(true)"
            class="px-3 py-1.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs transition active:scale-95"
          >
            立即检测
          </button>
        </div>
      </div>

      <!-- Active Ongoing 5v5 Game Panel -->
      <div v-else class="flex-1 flex flex-col overflow-hidden">
        <OngoingGamePanel
          :blue-team="activeBlueTeam"
          :red-team="activeRedTeam"
          :self-puuid="matchData?.my_puuid || (isDemoMode ? 'p-blue-1' : '')"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { Pin, RotateCw, X, Radio, AlertCircle } from 'lucide-vue-next'
import {
  getOngoingMatch,
  setOngoingOverlayVisible,
  setOngoingOverlayAlwaysOnTop,
  isOngoingOverlayVisible
} from '../api/tauri'
import { mockOngoingMatch } from '../api/mockData'
import type { OngoingMatchData, PlayerCard } from '../api/types'
import OngoingGamePanel from '../components/ongoing-game-panel/OngoingGamePanel.vue'

const isAlwaysOnTop = ref(true)
const isDemoMode = ref(false)
const isLoading = ref(false)
const matchData = ref<OngoingMatchData | null>(null)
let pollTimer: ReturnType<typeof setInterval> | null = null

const activeBlueTeam = computed<PlayerCard[]>(() => {
  if (isDemoMode.value) return mockOngoingMatch.blue_team
  return matchData.value?.blue_team || []
})

const activeRedTeam = computed<PlayerCard[]>(() => {
  if (isDemoMode.value) return mockOngoingMatch.red_team
  return matchData.value?.red_team || []
})

const statusColorClass = computed(() => {
  if (isDemoMode.value) return 'bg-amber-400 animate-pulse'
  if (matchData.value?.lcu_connected && matchData.value?.is_active) return 'bg-cyan-400 animate-pulse'
  if (matchData.value?.lcu_connected) return 'bg-emerald-400'
  return 'bg-slate-500'
})

const phaseLabel = computed(() => {
  if (isDemoMode.value) return '模拟演练中'
  if (!matchData.value?.lcu_connected) return '客户端离线'
  if (!matchData.value?.is_active) return '大厅等待中'
  if (matchData.value?.gameflow_phase === 'ChampSelect') return '英雄选择中'
  return matchData.value?.queue_name || '对局进行中'
})

async function loadMatchData(force = false) {
  if (isLoading.value) return
  if (isDemoMode.value && !force) return

  // Do not poll when overlay window is hidden unless forced by user click
  if (!force) {
    const isVisible = await isOngoingOverlayVisible()
    if (!isVisible) return
  }

  isLoading.value = true
  try {
    const res = await getOngoingMatch(false)
    if (res) {
      matchData.value = res
      // Auto-toggle demo mode off if real game starts
      if (res.is_active && isDemoMode.value) {
        isDemoMode.value = false
      }
    }
  } catch (err) {
    console.warn('Failed to load ongoing match for overlay:', err)
  } finally {
    isLoading.value = false
  }
}

function toggleDemoMode() {
  isDemoMode.value = !isDemoMode.value
  if (!isDemoMode.value) {
    loadMatchData(true)
  }
}

async function toggleAlwaysOnTop() {
  isAlwaysOnTop.value = !isAlwaysOnTop.value
  await setOngoingOverlayAlwaysOnTop(isAlwaysOnTop.value)
}

async function handleClose() {
  await setOngoingOverlayVisible(false)
}

onMounted(() => {
  loadMatchData()
  pollTimer = setInterval(() => {
    loadMatchData(false)
  }, 3000)
})

onUnmounted(() => {
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
})
</script>
