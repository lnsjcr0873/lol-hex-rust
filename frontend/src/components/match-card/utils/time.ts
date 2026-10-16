import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import 'dayjs/locale/zh-cn'

dayjs.extend(relativeTime)
dayjs.locale('zh-cn')

export function formatSeconds(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) return '00:00'
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
}

export function formatDuration(seconds: number): string {
  return formatSeconds(seconds)
}

export function formatMilliseconds(ms: number): string {
  if (!Number.isFinite(ms)) return '00:00'
  return formatSeconds(Math.floor(ms / 1000))
}

export function formatRelativeTime(timestamp: number): string {
  if (!Number.isFinite(timestamp) || timestamp <= 0) return '刚刚'
  // Auto-detect unix timestamp in seconds (10 digits) vs milliseconds (13 digits)
  const ts = timestamp < 10000000000 ? timestamp * 1000 : timestamp
  const now = Date.now()
  const diffSec = Math.floor((now - ts) / 1000)
  if (diffSec < 60) return '刚刚'
  if (diffSec < 3600) return `${Math.floor(diffSec / 60)}分钟前`
  if (diffSec < 86400) return `${Math.floor(diffSec / 3600)}小时前`
  if (diffSec < 86400 * 30) return `${Math.floor(diffSec / 86400)}天前`
  return dayjs(ts).format('YYYY-MM-DD')
}
