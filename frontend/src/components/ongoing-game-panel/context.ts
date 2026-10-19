import { InjectionKey, Ref, inject, provide, hasInjectionContext } from 'vue'
import type { EnhancedPlayerCard } from './types'

export interface OngoingGamePanelContext {
  kdaOutliers: Ref<Record<string, 'over' | 'below'>>
  selfPuuid: Ref<string>
  streamerMode: Ref<boolean>
  premadeTeamIdMap: Ref<Record<string, string>>
  selectPlayer: (player: EnhancedPlayerCard) => void
  inspectPlayer: (puuid: string) => void
  previewGame?: (gameId: number, puuid?: string) => void
}

export const OngoingGamePanelKey: InjectionKey<OngoingGamePanelContext> = Symbol('OngoingGamePanelKey')

export function useOngoingGamePanel(): OngoingGamePanelContext {
  const ctx = hasInjectionContext() ? inject(OngoingGamePanelKey, null) : null
  if (!ctx) {
    throw new Error('useOngoingGamePanel must be used within OngoingGamePanel')
  }
  return ctx
}

export function useOngoingGamePanelOptional(): OngoingGamePanelContext | null {
  if (!hasInjectionContext()) {
    return null
  }
  return inject(OngoingGamePanelKey, null) ?? null
}

export function provideOngoingGamePanel(context: OngoingGamePanelContext) {
  provide(OngoingGamePanelKey, context)
}
