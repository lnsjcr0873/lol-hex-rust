import { noZero } from '../../../utils'

export function avgOrZero(values: number[]): number {
  const finites = values.filter((v) => Number.isFinite(v))
  if (finites.length === 0) return 0
  return finites.reduce((s, v) => s + v, 0) / noZero(finites.length)
}

export function avgOrOne(values: number[]): number {
  const finites = values.filter((v) => Number.isFinite(v))
  if (finites.length === 0) return 1
  return finites.reduce((s, v) => s + v, 0) / noZero(finites.length)
}

export function avgOrNull(values: number[]): number | null {
  const finites = values.filter((v) => Number.isFinite(v))
  if (finites.length === 0) return null
  return finites.reduce((s, v) => s + v, 0) / finites.length
}

export function avgIfAllNonNull(values: (number | null)[]): number | null {
  if (values.length === 0) return null
  const allNonNull = values.every((v) => v !== null && Number.isFinite(v))
  if (!allNonNull) return null
  const nonNulls = values as number[]
  return nonNulls.reduce((s, v) => s + v, 0) / noZero(nonNulls.length)
}
