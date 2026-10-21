import { ref } from 'vue'
import { getNameFromYuriyuri } from '../../shared/utils/yuriyuri-names'

function getInitialStreamerMode(): boolean {
  try {
    const raw = typeof localStorage !== 'undefined' ? localStorage.getItem('hexbrain_preferences') : null
    if (raw) {
      const parsed = JSON.parse(raw)
      return Boolean(parsed?.streamerMode)
    }
  } catch {
    // ignore
  }
  return false
}

function getInitialAkariStyled(): boolean {
  try {
    const raw = typeof localStorage !== 'undefined' ? localStorage.getItem('hexbrain_preferences') : null
    if (raw) {
      const parsed = JSON.parse(raw)
      if (parsed?.streamerModeUseAkariStyledName !== undefined) {
        return Boolean(parsed.streamerModeUseAkariStyledName)
      }
    }
  } catch {
    // ignore
  }
  return true
}

const isStreamerMode = ref<boolean>(getInitialStreamerMode())
const streamerModeUseAkariStyledName = ref<boolean>(getInitialAkariStyled())

if (typeof window !== 'undefined') {
  const syncPreferences = () => {
    try {
      const raw = localStorage.getItem('hexbrain_preferences')
      if (raw) {
        const parsed = JSON.parse(raw)
        if (parsed?.streamerMode !== undefined) {
          isStreamerMode.value = Boolean(parsed.streamerMode)
        }
        if (parsed?.streamerModeUseAkariStyledName !== undefined) {
          streamerModeUseAkariStyledName.value = Boolean(parsed.streamerModeUseAkariStyledName)
        }
      }
    } catch {
      // ignore
    }
  }

  window.addEventListener('hexbrain-preferences-updated', syncPreferences)
  window.addEventListener('storage', syncPreferences)
}

export { getNameFromYuriyuri } from '../../shared/utils/yuriyuri-names'

export function streamerSummonerName(seedOrPuuid: string, index = 0): string {
  if (streamerModeUseAkariStyledName.value) {
    return getNameFromYuriyuri(seedOrPuuid || `player-${index}`, 'zh-CN')
  }
  return `召唤师 ${index + 1}`
}

export function masked(
  sensitiveText: string,
  replacer?: ((text: string) => string) | string
): string {
  if (typeof replacer === 'function') {
    return replacer(sensitiveText)
  } else if (typeof replacer === 'string') {
    return replacer
  }
  return '●●●●●●'
}

export function useStreamerModeMaskedText() {
  const setStreamerMode = (enabled: boolean) => {
    isStreamerMode.value = enabled
    try {
      const raw = localStorage.getItem('hexbrain_preferences')
      const parsed = raw ? JSON.parse(raw) : {}
      parsed.streamerMode = enabled
      const updated = JSON.stringify(parsed)
      localStorage.setItem('hexbrain_preferences', updated)
      window.dispatchEvent(new CustomEvent('hexbrain-preferences-updated', { detail: parsed }))
    } catch {
      // ignore
    }
  }

  const setStreamerModeUseAkariStyledName = (enabled: boolean) => {
    streamerModeUseAkariStyledName.value = enabled
    try {
      const raw = localStorage.getItem('hexbrain_preferences')
      const parsed = raw ? JSON.parse(raw) : {}
      parsed.streamerModeUseAkariStyledName = enabled
      const updated = JSON.stringify(parsed)
      localStorage.setItem('hexbrain_preferences', updated)
      window.dispatchEvent(new CustomEvent('hexbrain-preferences-updated', { detail: parsed }))
    } catch {
      // ignore
    }
  }

  const summonerName = (seedOrPuuid: string, index = 0): string => {
    if (!isStreamerMode.value) {
      return seedOrPuuid || '召唤师'
    }
    return streamerSummonerName(seedOrPuuid, index)
  }

  const maskedFn = (
    sensitiveText: string,
    replacer?: ((text: string) => string) | string
  ): string => {
    if (isStreamerMode.value) {
      return masked(sensitiveText, replacer)
    }
    return sensitiveText
  }

  return {
    isStreamerMode,
    streamerModeUseAkariStyledName,
    setStreamerMode,
    setStreamerModeUseAkariStyledName,
    summonerName,
    masked: maskedFn
  }
}
