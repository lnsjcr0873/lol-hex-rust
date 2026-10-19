<template>
  <div class="flex-1 flex flex-col h-full overflow-hidden bg-[#090d1a] relative select-none">
    <!-- Top Action Bar -->
    <div class="h-14 px-6 border-b border-[#162238] flex items-center justify-between shrink-0 bg-[#070b14]/90 backdrop-blur z-20">
      <div class="flex items-center space-x-3">
        <div class="flex items-center space-x-2">
          <span
            class="w-2.5 h-2.5 rounded-full"
            :class="[
              isDemoMode ? 'bg-amber-400 animate-pulse' :
              (matchData?.lcu_connected ? (matchData?.is_active ? 'bg-[#0ac8b9] animate-pulse' : 'bg-emerald-400') : 'bg-yellow-500/70')
            ]"
          ></span>
          <h2 class="text-base font-bold text-slate-100 tracking-wide">
            {{ isDemoMode ? '模拟演练对局 (5v5 严格分路对位演示)' : (matchData?.queue_name || '实时对局侦查') }}
          </h2>
        </div>

        <span
          v-if="isDemoMode"
          class="px-2 py-0.5 rounded text-xs bg-amber-950/80 border border-amber-600/50 text-amber-300 font-medium"
        >
          演练模式
        </span>
        <span
          v-else-if="matchData?.lcu_connected && matchData?.is_active"
          class="px-2 py-0.5 rounded text-xs bg-[#101b2e] border border-[#1e2e4a] text-[#0ac8b9] font-medium"
        >
          {{ matchData?.gameflow_phase === 'ChampSelect' ? '英雄选择中' : '进行中' }}
        </span>
        <span
          v-else-if="matchData?.lcu_connected"
          class="px-2 py-0.5 rounded text-xs bg-emerald-950/60 border border-emerald-600/40 text-emerald-300 font-medium"
        >
          客户端在线
        </span>
        <span
          v-else
          class="px-2 py-0.5 rounded text-xs bg-slate-800/80 border border-slate-700 text-slate-400 font-medium"
        >
          未连接
        </span>
      </div>

      <!-- Quick Actions -->
      <div class="flex items-center space-x-3">
        <!-- Chat Record Broadcast Options -->
        <div v-if="(matchData?.is_active || isDemoMode)" class="flex items-center space-x-1.5">
          <button
            @click="sendChat('concise')"
            :disabled="isBroadcasting"
            class="px-3 py-1.5 rounded-lg bg-[#0e273c] hover:bg-[#143452] border border-[#1b4366] text-[#38bdf8] text-xs font-semibold flex items-center space-x-1.5 transition active:scale-95 disabled:opacity-50 shadow-sm"
            title="将我方 5 人近期胜率与战绩以精简格式发送到选人聊天室"
          >
            <MessageSquare class="w-3.5 h-3.5" />
            <span>{{ isBroadcasting ? '广播中...' : '广播精简战绩' }}</span>
          </button>

          <button
            @click="sendChat('detailed')"
            :disabled="isBroadcasting"
            class="px-2.5 py-1.5 rounded-lg bg-[#0d2133] hover:bg-[#122e47] border border-[#183955] text-slate-300 hover:text-white text-xs font-medium transition active:scale-95 disabled:opacity-50"
            title="发送包含段位、胜率与车队标记的详细战绩"
          >
            <span>详细版</span>
          </button>
        </div>

        <!-- Demo Mode Toggle -->
        <button
          @click="toggleDemoMode"
          class="px-3 py-1.5 rounded-lg border text-xs font-medium flex items-center space-x-1.5 transition active:scale-95"
          :class="isDemoMode ? 'bg-amber-950/80 hover:bg-amber-900 border-amber-600 text-amber-200' : 'bg-[#0f172a] hover:bg-[#162238] border-[#1e293b] text-slate-300'"
        >
          <Gamepad2 class="w-3.5 h-3.5" />
          <span>{{ isDemoMode ? '退出演练 (切回实时)' : '模拟演练模式' }}</span>
        </button>

        <!-- Standalone Floating Window Toggle -->
        <button
          @click="handleToggleOverlay"
          class="px-3 py-1.5 rounded-lg border border-cyan-500/40 bg-cyan-950/40 hover:bg-cyan-900/60 text-cyan-300 text-xs font-semibold flex items-center space-x-1.5 transition active:scale-95 shadow-sm"
          title="弹出游戏内置顶独立对局悬浮窗 (支持拖拽/贴边/透明置顶)"
        >
          <ExternalLink class="w-3.5 h-3.5" />
          <span>独立悬浮窗</span>
        </button>

        <button
          @click="refreshData"
          class="p-2 rounded-lg bg-[#0f172a] hover:bg-[#162238] border border-[#1e293b] text-slate-300 hover:text-white text-xs transition"
          title="强制重新感知 LCU 对局"
        >
          <RotateCw class="w-3.5 h-3.5" :class="{ 'animate-spin': isRefreshing }" />
        </button>
      </div>
    </div>

    <!-- Notification Toast -->
    <div
      v-if="toastMessage"
      class="absolute top-16 right-6 z-50 px-4 py-2 rounded-xl bg-slate-900/95 border border-cyan-500/50 text-cyan-300 text-xs shadow-2xl flex items-center space-x-2 transition-all duration-300"
    >
      <span>✓</span>
      <span>{{ toastMessage }}</span>
    </div>

    <!-- State 1: Offline / Lobby Empty State View -->
    <div
      v-if="!isDemoMode && (!matchData?.lcu_connected || !matchData?.is_active)"
      class="flex-1 flex flex-col items-center justify-center p-8 text-center"
    >
      <div class="max-w-md p-8 rounded-2xl bg-[#0c1424]/90 border border-[#192b4a] shadow-2xl flex flex-col items-center relative overflow-hidden backdrop-blur">
        <!-- Ambient Glow -->
        <div class="absolute -top-16 -right-16 w-32 h-32 bg-[#0ac8b9]/10 rounded-full blur-2xl pointer-events-none"></div>
        <div class="absolute -bottom-16 -left-16 w-32 h-32 bg-blue-600/10 rounded-full blur-2xl pointer-events-none"></div>

        <div class="w-16 h-16 rounded-2xl bg-[#101b30] border border-[#1e2e4a] flex items-center justify-center mb-5 text-[#0ac8b9] shadow-inner">
          <Radio v-if="matchData?.lcu_connected" class="w-8 h-8 animate-pulse text-emerald-400" />
          <AlertCircle v-else class="w-8 h-8 text-amber-400" />
        </div>

        <h3 class="text-lg font-bold text-slate-100 mb-2">
          {{ matchData?.lcu_connected ? '英雄联盟客户端已连接 · 等待对局开始' : '等待英雄联盟客户端启动' }}
        </h3>

        <p class="text-xs text-slate-400 leading-relaxed mb-6">
          {{ matchData?.lcu_connected
              ? '当前处于大厅或组队状态。进入【英雄选择阶段】或【加载对局】后，HexBrain 将自动实时穿透展示 10 人实战战绩、车队开黑标记与 5v5 严格分路对位。'
              : '后台已开启实时嗅探。请启动英雄联盟并登录账号，系统将即时捕获对局信息，无需手动配置。'
          }}
        </p>

        <div class="flex items-center space-x-3 w-full">
          <button
            @click="toggleDemoMode"
            class="flex-1 px-4 py-2.5 rounded-xl bg-gradient-to-r from-[#0ac8b9]/20 to-blue-600/20 hover:from-[#0ac8b9]/30 hover:to-blue-600/30 border border-[#0ac8b9]/40 text-[#0ac8b9] hover:text-white text-xs font-semibold flex items-center justify-center space-x-2 transition active:scale-95 shadow-lg"
          >
            <Sparkles class="w-4 h-4" />
            <span>开启演练模拟模式</span>
          </button>

          <button
            @click="refreshData"
            class="px-4 py-2.5 rounded-xl bg-[#121c2e] hover:bg-[#1a2942] border border-[#233552] text-slate-300 text-xs font-medium transition active:scale-95"
          >
            立即检测
          </button>
        </div>

        <div class="mt-4 flex items-center space-x-1.5 text-[11px] text-slate-500">
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-ping"></span>
          <span>心跳嗅探进程运行中 · 每 2.5 秒自动同步</span>
        </div>
      </div>
    </div>

    <!-- State 2: Active 5v5 Ongoing Game Panel View -->
    <div v-else class="flex-1 flex flex-col overflow-hidden">
      <!-- Demo Mode Banner -->
      <div
        v-if="isDemoMode"
        class="h-8 px-6 bg-amber-950/60 border-b border-amber-600/30 flex items-center justify-between text-xs text-amber-300/90 shrink-0"
      >
        <span class="flex items-center space-x-1.5">
          <span>⚠️</span>
          <span>当前处于【模拟演练模式】：展示 5v5 严格分路横向对位 (上单对上单、打野对打野等) 与多排车队高亮效果</span>
        </span>
        <button
          @click="toggleDemoMode"
          class="text-[11px] underline hover:text-amber-100 transition"
        >
          退出演练回到实时监听
        </button>
      </div>

      <!-- Assemble OngoingGamePanel Component -->
      <div class="flex-1 overflow-hidden">
        <OngoingGamePanel
          :blue-team="activeBlueTeam"
          :red-team="activeRedTeam"
          :self-puuid="matchData?.my_puuid || (isDemoMode ? 'p-blue-1' : '')"
          :selected-puuid="selectedPlayer?.puuid"
          @select-player="handlePlayerSelect"
          @inspect-player="handlePlayerInspect"
          @preview-game="handlePreviewGame"
        />
      </div>
    </div>

    <!-- Connected Match Previewer Modal (1:1 with LeagueAkari) -->
    <ConnectedMatchPreviewer
      v-model:show="showPreviewModal"
      :game-id="previewGameId"
      :puuid="previewPuuid"
      @navigate-to-summoner-by-puuid="handleInspectHistory"
    />

    <!-- Player Detail Drawer -->
    <PlayerDetailDrawer
      v-if="selectedPlayer"
      :player="selectedPlayer"
      @close="selectedPlayer = null"
      @inspect-history="handleInspectHistory"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import {
  MessageSquare,
  Gamepad2,
  RotateCw,
  Radio,
  AlertCircle,
  Sparkles,
  ExternalLink
} from 'lucide-vue-next'
import type { OngoingMatchData, PlayerCard } from '../api/types'
import { getOngoingMatch, sendChatRecords, toggleOngoingOverlay } from '../api/tauri'
import { mockOngoingMatch } from '../api/mockData'
import OngoingGamePanel from '../components/ongoing-game-panel/OngoingGamePanel.vue'
import PlayerDetailDrawer from '../components/PlayerDetailDrawer.vue'
import ConnectedMatchPreviewer from '../components/match-preview/ConnectedMatchPreviewer.vue'
import type { EnhancedPlayerCard } from '../components/ongoing-game-panel/types'

const matchData = ref<OngoingMatchData | null>(null)
const isDemoMode = ref(false)
const isRefreshing = ref(false)
const isBroadcasting = ref(false)
const selectedPlayer = ref<EnhancedPlayerCard | null>(null)
const toastMessage = ref('')
const showPreviewModal = ref(false)
const previewGameId = ref(0)
const previewPuuid = ref('')
let pollTimer: number | null = null

const handlePreviewGame = (payload: { gameId: number; puuid?: string }) => {
  previewGameId.value = payload.gameId
  previewPuuid.value = payload.puuid || ''
  showPreviewModal.value = true
}

const activeBlueTeam = computed<PlayerCard[]>(() => {
  if (isDemoMode.value) {
    return mockOngoingMatch.blue_team
  }
  return matchData.value?.blue_team || []
})

const activeRedTeam = computed<PlayerCard[]>(() => {
  if (isDemoMode.value) {
    return mockOngoingMatch.red_team
  }
  return matchData.value?.red_team || []
})

let toastTimer: number | null = null
const showToast = (msg: string) => {
  if (toastTimer) clearTimeout(toastTimer)
  toastMessage.value = msg
  toastTimer = window.setTimeout(() => {
    toastMessage.value = ''
    toastTimer = null
  }, 2800)
}

const refreshData = async () => {
  if (isDemoMode.value) {
    matchData.value = mockOngoingMatch
    return
  }
  isRefreshing.value = true
  try {
    const data = await getOngoingMatch(false)
    if (!isDemoMode.value) {
      matchData.value = data
    }
  } catch (err) {
    console.error('Failed to get ongoing match:', err)
  } finally {
    isRefreshing.value = false
  }
}

const toggleDemoMode = () => {
  isDemoMode.value = !isDemoMode.value
  selectedPlayer.value = null
  if (isDemoMode.value) {
    matchData.value = mockOngoingMatch
    showToast('已进入演练模式')
  } else {
    refreshData()
    showToast('已切换回实时监听')
  }
}

const sendChat = async (format: 'concise' | 'detailed') => {
  if (isBroadcasting.value) return
  isBroadcasting.value = true
  try {
    const roomId = matchData.value?.chat_room_id || ''
    const res = await sendChatRecords(roomId, format)
    showToast(res.message || '战绩已广播至聊天室')
  } catch (err) {
    console.error('Chat broadcast error:', err)
    showToast('战绩广播发送失败，请确认是否处于选人阶段')
  } finally {
    isBroadcasting.value = false
  }
}

const emit = defineEmits<{
  (e: 'navigate-to-summoner', puuid: string): void
}>()

const handlePlayerSelect = (player: EnhancedPlayerCard) => {
  selectedPlayer.value = player
}

const handlePlayerInspect = (puuid: string) => {
  emit('navigate-to-summoner', puuid)
}

const handleInspectHistory = (puuid: string) => {
  emit('navigate-to-summoner', puuid)
}

const handleToggleOverlay = async () => {
  try {
    await toggleOngoingOverlay()
  } catch (e) {
    console.warn('Failed to toggle ongoing overlay:', e)
  }
}

let isPolling = false

onMounted(() => {
  refreshData()
  pollTimer = window.setInterval(async () => {
    if (!isDemoMode.value && !isPolling) {
      isPolling = true
      try {
        const data = await getOngoingMatch(false)
        if (!isDemoMode.value) {
          matchData.value = data
        }
      } catch {
        // Suppress background poll errors
      } finally {
        isPolling = false
      }
    }
  }, 2500)
})

onUnmounted(() => {
  if (pollTimer) {
    clearInterval(pollTimer)
  }
  if (toastTimer) {
    clearTimeout(toastTimer)
  }
})

defineExpose({
  refreshData
})
</script>
