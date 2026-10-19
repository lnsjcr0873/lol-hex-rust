import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { getLcuStatus } from '../api/tauri'
import type { LcuStatus } from '../api/types'

export const useLcuStore = defineStore('lcu', () => {
  const isConnected = ref(false)
  const gameflowPhase = ref('None')
  const puuid = ref('')
  const gameName = ref('')
  const tagLine = ref('')
  const profileIconId = ref(0)
  const summonerLevel = ref(0)
  const isLoading = ref(false)
  const lastUpdated = ref<number | null>(null)

  let pollTimer: any = null
  let subscriberCount = 0

  const displayName = computed(() => {
    if (!gameName.value) return ''
    return tagLine.value ? `${gameName.value}#${tagLine.value}` : gameName.value
  })

  const connectionDisplayText = computed(() => {
    if (isConnected.value) {
      return displayName.value ? `${displayName.value} (已连接)` : 'LCU 已连接'
    }
    return 'LCU 离线 (请登录游戏)'
  })

  const phaseDisplayText = computed(() => {
    if (!isConnected.value) {
      return '客户端未连接'
    }
    switch (gameflowPhase.value) {
      case 'ChampSelect':
        return '英雄选择进行中'
      case 'GameStart':
        return '对局加载中'
      case 'InProgress':
        return '正在游戏中'
      case 'ReadyCheck':
        return '对局就绪 (待确认)'
      case 'Matchmaking':
        return '匹配中'
      case 'Lobby':
        return '大厅待命中'
      case 'EndOfGame':
      case 'PreEndOfGame':
      case 'WaitingForStats':
        return '对局结算中'
      case 'None':
      default:
        return '大厅待命中'
    }
  })

  async function updateStatus(): Promise<LcuStatus | null> {
    isLoading.value = true
    try {
      const status = await getLcuStatus()
      isConnected.value = Boolean(status?.is_connected)
      gameflowPhase.value = status?.gameflow_phase || 'None'
      puuid.value = status?.puuid || ''
      gameName.value = status?.game_name || ''
      tagLine.value = status?.tag_line || ''
      profileIconId.value = status?.profile_icon_id || 0
      summonerLevel.value = status?.summoner_level || 0
      lastUpdated.value = Date.now()
      return status
    } catch {
      isConnected.value = false
      gameflowPhase.value = 'None'
      puuid.value = ''
      gameName.value = ''
      tagLine.value = ''
      return null
    } finally {
      isLoading.value = false
    }
  }

  function startPolling(intervalMs = 3000) {
    subscriberCount++
    if (!pollTimer) {
      updateStatus()
      pollTimer = setInterval(updateStatus, intervalMs)
    }
  }

  function stopPolling() {
    subscriberCount = Math.max(0, subscriberCount - 1)
    if (subscriberCount === 0 && pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
  }

  return {
    isConnected,
    gameflowPhase,
    puuid,
    gameName,
    tagLine,
    profileIconId,
    summonerLevel,
    isLoading,
    lastUpdated,
    displayName,
    connectionDisplayText,
    phaseDisplayText,
    updateStatus,
    startPolling,
    stopPolling
  }
})
