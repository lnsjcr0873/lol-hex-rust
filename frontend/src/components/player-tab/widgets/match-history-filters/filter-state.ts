import type { Predicate, GameScope, ParticipantScope } from '../../../../shared/data-adapter/predicates/combinators'
import {
  and,
  or,
  not,
  game,
  player,
  anyone,
  everyone,
  allies,
  enemies,
  isWin,
  isLoss,
  isRemake,
  isChampion,
  isPosition,
  isQueue,
  hasItem,
  hasPerk,
  hasPlayer,
  gameCreationInTimeRange,
  killsBetween,
  deathsBetween,
  assistsBetween,
  kdaBetween,
  damageDealtToChampionsBetween,
  damageTakenBetween,
  goldBetween,
  csBetween,
  killParticipationBetween
} from '../../../../shared/data-adapter/predicates/combinators'
import type { LcuOrSgpGameSummary } from '../../../../shared/data-adapter/wrapper'

export type SimpleWinLoss = 'all' | 'win' | 'loss'
export type SimpleTimeRange =
  | 'all'
  | 'last3Hours'
  | 'last12Hours'
  | 'last24Hours'
  | 'last3Days'
  | 'last7Days'
  | 'last30Days'

export interface SimpleMatchHistoryFilterState {
  winLoss: SimpleWinLoss
  timeRange: SimpleTimeRange
  positions: string[]
  championIds: number[]
  summonerPuuids: string[]
}

export function createDefaultSimpleFilter(): SimpleMatchHistoryFilterState {
  return {
    winLoss: 'all',
    timeRange: 'all',
    positions: [],
    championIds: [],
    summonerPuuids: []
  }
}

export interface AstNode {
  id: string
  type:
    | 'and'
    | 'or'
    | 'not'
    | 'isWin'
    | 'isLoss'
    | 'isChampion'
    | 'isPosition'
    | 'hasItem'
    | 'hasPerk'
    | 'hasPlayer'
    | 'numberBetween'
    | 'scope'
  params?: Record<string, any>
  children?: string[] // child node IDs
}

export interface AdvancedFilterState {
  rootId: string
  nodeMap: Record<string, AstNode>
}

export function createDefaultAdvancedFilter(): AdvancedFilterState {
  const rootId = 'node_root'
  return {
    rootId,
    nodeMap: {
      [rootId]: {
        id: rootId,
        type: 'and',
        children: []
      }
    }
  }
}

export function hasActiveFilter(
  simple: SimpleMatchHistoryFilterState,
  advanced: AdvancedFilterState,
  mode: 'simple' | 'advanced'
): boolean {
  if (mode === 'simple') {
    return (
      simple.winLoss !== 'all' ||
      simple.timeRange !== 'all' ||
      simple.positions.length > 0 ||
      simple.championIds.length > 0 ||
      simple.summonerPuuids.length > 0
    )
  }
  const root = advanced.nodeMap[advanced.rootId]
  return Boolean(root && root.children && root.children.length > 0)
}

/**
 * Converts a SimpleMatchHistoryFilterState into an executable combinator predicate
 */
export function buildPredicateFromSimple(
  state: SimpleMatchHistoryFilterState,
  selfPuuid?: string
): Predicate<LcuOrSgpGameSummary> {
  const predicates: Predicate<GameScope>[] = []

  // 1. Time range
  if (state.timeRange && state.timeRange !== 'all') {
    predicates.push(gameCreationInTimeRange(state.timeRange))
  }

  // 2. Win / Loss
  if (state.winLoss === 'win') {
    if (selfPuuid) {
      predicates.push(player(selfPuuid, isWin()) as any)
    } else {
      predicates.push(anyone(isWin()) as any)
    }
  } else if (state.winLoss === 'loss') {
    if (selfPuuid) {
      predicates.push(
        player(selfPuuid, or(isLoss(false), isLoss(true))) as any
      )
    } else {
      predicates.push(anyone(or(isLoss(false), isLoss(true))) as any)
    }
  }

  // 3. Position / Lane
  if (state.positions && state.positions.length > 0) {
    const posPreds = state.positions.map((pos) => isPosition(pos))
    const combinedPos = posPreds.length === 1 ? posPreds[0] : or(...posPreds)
    if (selfPuuid) {
      predicates.push(player(selfPuuid, combinedPos) as any)
    } else {
      predicates.push(anyone(combinedPos) as any)
    }
  }

  // 4. Champion
  if (state.championIds && state.championIds.length > 0) {
    const champPreds = state.championIds.map((cid) => isChampion(cid))
    const combinedChamp = champPreds.length === 1 ? champPreds[0] : or(...champPreds)
    if (selfPuuid) {
      predicates.push(player(selfPuuid, combinedChamp) as any)
    } else {
      predicates.push(anyone(combinedChamp) as any)
    }
  }

  // 5. Summoners / Teammates
  if (state.summonerPuuids && state.summonerPuuids.length > 0) {
    for (const p of state.summonerPuuids) {
      predicates.push(hasPlayer(p) as any)
    }
  }

  if (predicates.length === 0) {
    return () => true
  }

  return game(and(...predicates))
}

/**
 * Converts an AST Node Map into an executable combinator predicate
 */
export function buildPredicateFromAst(
  state?: AdvancedFilterState | null,
  selfPuuid?: string
): Predicate<LcuOrSgpGameSummary> {
  if (!state || !state.nodeMap) {
    return () => true
  }
  const { rootId, nodeMap } = state
  const root = nodeMap[rootId]
  if (!root || !root.children || root.children.length === 0) {
    return () => true
  }

  function compileParticipantNode(node: AstNode, visited: Set<string> = new Set()): Predicate<ParticipantScope> {
    if (!node || visited.has(node.id)) return () => true
    visited.add(node.id)
    switch (node.type) {
      case 'isWin':
        return isWin()
      case 'isLoss':
        return or(isLoss(false), isLoss(true))
      case 'isChampion':
        return isChampion(Number(node.params?.championId || 0))
      case 'isPosition':
        return isPosition(String(node.params?.position || ''))
      case 'hasItem':
        return hasItem(Number(node.params?.itemId || 0))
      case 'hasPerk':
        return hasPerk(Number(node.params?.perkId || 0))
      case 'numberBetween': {
        const metric = String(node.params?.metric || 'kills')
        const rawMin = Number(node.params?.min)
        const rawMax = Number(node.params?.max)
        const min = Number.isFinite(rawMin) ? rawMin : 0
        const max = Number.isFinite(rawMax) ? rawMax : Infinity
        switch (metric) {
          case 'kills':
            return killsBetween(min, max)
          case 'deaths':
            return deathsBetween(min, max)
          case 'assists':
            return assistsBetween(min, max)
          case 'kda':
            return kdaBetween(min, max)
          case 'damage':
            return damageDealtToChampionsBetween(min, max)
          case 'damageTaken':
            return damageTakenBetween(min, max)
          case 'gold':
            return goldBetween(min, max)
          case 'cs':
            return csBetween(min, max)
          case 'killParticipation':
            return killParticipationBetween(min, max)
          default:
            return () => true
        }
      }
      case 'not': {
        const childId = node.children?.[0]
        if (!childId || !nodeMap[childId]) return () => true
        return not(compileParticipantNode(nodeMap[childId], new Set(visited)))
      }
      case 'and': {
        const childPreds = (node.children || [])
          .map((id) => nodeMap[id])
          .filter(Boolean)
          .map((child) => compileParticipantNode(child, new Set(visited)))
        return childPreds.length > 0 ? and(...childPreds) : () => true
      }
      case 'or': {
        const childPreds = (node.children || [])
          .map((id) => nodeMap[id])
          .filter(Boolean)
          .map((child) => compileParticipantNode(child, new Set(visited)))
        return childPreds.length > 0 ? or(...childPreds) : () => true
      }
      default:
        return () => true
    }
  }

  function compileGameNode(node: AstNode, visited: Set<string> = new Set()): Predicate<GameScope> {
    if (!node || visited.has(node.id)) return () => true
    visited.add(node.id)
    switch (node.type) {
      case 'hasPlayer':
        return hasPlayer(String(node.params?.puuid || '')) as any
      case 'scope': {
        const scopeType = node.params?.scope || 'self'
        const childId = node.children?.[0]
        if (!childId || !nodeMap[childId]) return () => true
        const partPred = compileParticipantNode(nodeMap[childId], new Set())
        if (scopeType === 'self' && selfPuuid) {
          return player(selfPuuid, partPred) as any
        }
        if (scopeType === 'allies' && selfPuuid) {
          return allies(selfPuuid, (scope) => (scope?.participants || []).some((p) => partPred({ context: scope.context, participant: p }))) as any
        }
        if (scopeType === 'enemies' && selfPuuid) {
          return enemies(selfPuuid, (scope) => (scope?.participants || []).some((p) => partPred({ context: scope.context, participant: p }))) as any
        }
        return anyone(partPred) as any
      }
      case 'and': {
        const childPreds = (node.children || [])
          .map((id) => nodeMap[id])
          .filter(Boolean)
          .map((child) => {
            if (['isWin', 'isLoss', 'isChampion', 'isPosition', 'hasItem', 'hasPerk', 'numberBetween'].includes(child.type)) {
              const partPred = compileParticipantNode(child, new Set())
              return selfPuuid ? (player(selfPuuid, partPred) as any) : (anyone(partPred) as any)
            }
            return compileGameNode(child, new Set(visited))
          })
        return childPreds.length > 0 ? and(...childPreds) : () => true
      }
      case 'or': {
        const childPreds = (node.children || [])
          .map((id) => nodeMap[id])
          .filter(Boolean)
          .map((child) => {
            if (['isWin', 'isLoss', 'isChampion', 'isPosition', 'hasItem', 'hasPerk', 'numberBetween'].includes(child.type)) {
              const partPred = compileParticipantNode(child, new Set())
              return selfPuuid ? (player(selfPuuid, partPred) as any) : (anyone(partPred) as any)
            }
            return compileGameNode(child, new Set(visited))
          })
        return childPreds.length > 0 ? or(...childPreds) : () => true
      }
      case 'not': {
        const childId = node.children?.[0]
        if (!childId || !nodeMap[childId]) return () => true
        return not(compileGameNode(nodeMap[childId], new Set(visited)))
      }
      default: {
        const partPred = compileParticipantNode(node, new Set())
        return selfPuuid ? (player(selfPuuid, partPred) as any) : (anyone(partPred) as any)
      }
    }
  }

  return game(compileGameNode(root, new Set()))
}
