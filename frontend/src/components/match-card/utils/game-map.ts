export const LOL_MAP_DOMAINS: Record<number, { minX: number; minY: number; maxX: number; maxY: number }> = {
  1: { minX: -650, minY: -83, maxX: 14076, maxY: 14522 },
  11: { minX: 0, minY: 0, maxX: 14820, maxY: 14881 },
  12: { minX: -28, minY: -19, maxX: 12849, maxY: 12858 },
  21: { minX: 0, minY: 0, maxX: 15e3, maxY: 15e3 }
}

export type MapId = 11 | 12 | 21

export function getMapDomain(mapId: number = 11) {
  return LOL_MAP_DOMAINS[mapId] || LOL_MAP_DOMAINS[11]
}

export function mapToImagePosition(
  x: number,
  y: number,
  imageWidth: number,
  imageHeight: number,
  mapId: number = 11,
  options?: { invertY?: boolean; clamp?: boolean }
) {
  if (!Number.isFinite(x) || !Number.isFinite(y)) {
    return { left: 0, top: 0 }
  }

  const { invertY = true, clamp = true } = options || {}
  const domain = getMapDomain(mapId)

  const rangeX = domain.maxX - domain.minX
  const rangeY = domain.maxY - domain.minY

  if (rangeX === 0 || rangeY === 0) {
    return { left: 0, top: 0 }
  }

  const normalizedX = (x - domain.minX) / rangeX
  const normalizedY = invertY ? (domain.maxY - y) / rangeY : (y - domain.minY) / rangeY

  let left = normalizedX * imageWidth
  let top = normalizedY * imageHeight

  if (!Number.isFinite(left)) left = 0
  if (!Number.isFinite(top)) top = 0

  if (clamp) {
    if (left < 0) left = 0
    if (top < 0) top = 0
    if (left > imageWidth) left = imageWidth
    if (top > imageHeight) top = imageHeight
  }

  return { left, top }
}

export function isSupportedMap(mapId: number) {
  return [11, 12, 21].includes(mapId)
}
