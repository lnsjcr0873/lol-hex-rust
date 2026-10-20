type Edge = {
  to: string
  weight: number
  ids: Set<string>
}

type ResultType = { players: string[]; times: number; ids: Set<string> }

/**
 * 用于高效分析预组队情况的图结构
 */
export class TeamUpGraph {
  private _adjacencyList: Map<string, Map<string, Edge>>

  constructor() {
    this._adjacencyList = new Map()
  }

  addVertex(vertex: string): void {
    if (!this._adjacencyList.has(vertex)) {
      this._adjacencyList.set(vertex, new Map())
    }
  }

  addEdge(vertex1: string, vertex2: string, id: string): void {
    this.addVertex(vertex1)
    this.addVertex(vertex2)

    let list1 = this._adjacencyList.get(vertex1) || new Map()
    let list2 = this._adjacencyList.get(vertex2) || new Map()

    let edge1 = list1.get(vertex2)
    if (edge1) {
      edge1.weight++
      edge1.ids.add(id)
    } else {
      list1.set(vertex2, { to: vertex2, weight: 1, ids: new Set([id]) })
    }

    let edge2 = list2.get(vertex1)
    if (edge2) {
      edge2.weight++
      edge2.ids.add(id)
    } else {
      list2.set(vertex1, { to: vertex1, weight: 1, ids: new Set([id]) })
    }
  }

  getTogetherTimes(playersGroups: string[][]): ResultType[] {
    const results: ResultType[] = []

    for (const players of playersGroups) {
      if (players.length < 2) continue
      let intersectionIds: Set<string> | null = null

      for (let i = 0; i < players.length; i++) {
        for (let j = i + 1; j < players.length; j++) {
          const adjacencyMap = this._adjacencyList.get(players[i])
          if (!adjacencyMap) {
            intersectionIds = new Set()
            break
          }

          const edge = adjacencyMap.get(players[j])
          if (!edge) {
            intersectionIds = new Set()
            break
          }

          if (intersectionIds === null) {
            intersectionIds = new Set(edge.ids)
          } else {
            intersectionIds = new Set([...intersectionIds].filter((id) => edge.ids.has(id)))
          }
        }
        if (intersectionIds && intersectionIds.size === 0) break
      }

      const count = intersectionIds ? intersectionIds.size : 0
      results.push({ players, times: count, ids: intersectionIds || new Set() })
    }

    return results
  }
}

type ElementKey = number | string

/**
 * 😡💢
 */
export class UnionFind {
  private parent: Map<ElementKey, ElementKey>
  private rank: Map<ElementKey, number>

  constructor(elements: ElementKey[]) {
    this.parent = new Map()
    this.rank = new Map()
    elements.forEach((element) => {
      this.parent.set(element, element)
      this.rank.set(element, 0)
    })
  }

  find(element: ElementKey): ElementKey {
    if (element === undefined || element === null || !this.parent.has(element)) {
      return element
    }
    if (this.parent.get(element) !== element) {
      this.parent.set(element, this.find(this.parent.get(element)!))
    }
    return this.parent.get(element)!
  }

  union(x: ElementKey, y: ElementKey) {
    let rootX = this.find(x)
    let rootY = this.find(y)

    if (rootX !== rootY) {
      let rankX = this.rank.get(rootX)!
      let rankY = this.rank.get(rootY)!

      if (rankX > rankY) {
        this.parent.set(rootY, rootX)
      } else if (rankX < rankY) {
        this.parent.set(rootX, rootY)
      } else {
        this.parent.set(rootY, rootX)
        this.rank.set(rootX, rankX + 1)
      }
    }
  }
}

export function combinations(nums: string[], maxK = 5): string[][] {
  const safeNums = nums.length > 8 ? nums.slice(0, 8) : nums
  const res: string[][] = []

  function backtrack(start: number, current: string[]) {
    if (current.length > maxK) return
    res.push([...current])

    for (let i = start; i < safeNums.length; i++) {
      current.push(safeNums[i])
      backtrack(i + 1, current)
      current.pop()
    }
  }

  backtrack(0, [])
  return res
}

/**
 * @param matches 包含了若干对局的召唤师 ID 列表
 * @param players 需要计算的玩家列表，时间复杂度很高，所以要尽可能少
 * @param threshold 阈值，多少重复场次即过滤
 * @returns
 */
export function calculateTogetherTimes(
  matches: { players: string[]; id: string }[],
  players: string[],
  threshold = 3
): Array<{ players: string[]; times: number; ids: string[] }> {
  const graph = new TeamUpGraph()
  const set = new Set(players)

  matches
    .map((m) => ({
      players: m.players.filter((mm) => set.has(mm)),
      id: m.id
    }))
    .forEach((m) => {
      for (let i = 0; i < m.players.length - 1; i++) {
        for (let j = i + 1; j < m.players.length; j++) {
          graph.addEdge(m.players[i], m.players[j], m.id)
        }
      }
    })

  const result = graph
    .getTogetherTimes(combinations(players).filter((a) => a.length > 1))
    .filter((t) => t.times >= threshold)

  // Sort candidate cliques descending by team size then by times
  const sorted = result.slice().sort((a, b) => b.players.length - a.players.length || b.times - a.times)
  const finalResults: ResultType[] = []

  for (const r of sorted) {
    const rPlayerSet = new Set(r.players)
    // Check if this group is already subsumed by an existing larger group that played the same or superset of matches
    const isSubsumed = finalResults.some((existing) => {
      if (isSuperset(new Set(existing.players), rPlayerSet)) {
        return isSuperset(existing.ids, r.ids)
      }
      return false
    })

    if (!isSubsumed) {
      finalResults.push(r)
    }
  }

  return finalResults.map((v) => ({
    ids: Array.from(v.ids),
    players: v.players,
    times: v.times
  }))
}

function toSetKey(set: Set<string>) {
  return Array.from(set.values()).sort().join('|')
}

function isSuperset(set: Set<string>, subset: Set<string>) {
  if (subset.size > set.size) {
    return false
  }

  for (const elem of subset.values()) {
    if (!set.has(elem)) {
      return false
    }
  }

  return true
}

export function removeSubsets<T>(objects: T[], getter: (element: T) => ElementKey[]): T[] {
  const sets = objects.map((obj) => new Set(getter(obj)))

  const isSubsetOrEqual = (a: Set<ElementKey>, b: Set<ElementKey>) =>
    Array.from(a).every((element) => b.has(element))

  return objects.filter((_object, index) => {
    const currentSet = sets[index]

    return !sets.some((otherSet, otherIndex) => {
      if (index === otherIndex || otherSet.size < currentSet.size) {
        return false
      }

      if (otherSet.size === currentSet.size) {
        return otherIndex < index && isSubsetOrEqual(currentSet, otherSet)
      }

      return isSubsetOrEqual(currentSet, otherSet)
    })
  })
}

export function mergeOverlappingSets(keys: ElementKey[][]): ElementKey[][] {
  const validKeys = (keys || []).filter((k) => k && k.length > 0)
  if (validKeys.length === 0) return []

  const elements = new Set<ElementKey>()
  validKeys.forEach((k) => k.forEach((el) => elements.add(el)))
  const uf = new UnionFind(Array.from(elements))

  validKeys.forEach((k) => {
    for (let i = 0; i < k.length; i++) {
      for (let j = i + 1; j < k.length; j++) {
        uf.union(k[i], k[j])
      }
    }
  })

  const unionMap = new Map<ElementKey, Set<ElementKey>>()
  validKeys.forEach((k) => {
    const root = uf.find(k[0])
    if (!unionMap.has(root)) {
      unionMap.set(root, new Set<ElementKey>())
    }
    k.forEach((el) => unionMap.get(root)!.add(el))
  })

  return Array.from(unionMap.values()).map((set) => Array.from(set))
}
