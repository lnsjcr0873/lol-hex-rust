import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import 'dayjs/locale/zh-cn'

dayjs.extend(relativeTime)
dayjs.locale('zh-cn')

export function formatSeconds(seconds: number): string {
  if (isNaN(seconds) || seconds < 0) return '00:00'
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
}

export function formatDuration(seconds: number): string {
  return formatSeconds(seconds)
}

export function formatMilliseconds(ms: number): string {
  return formatSeconds(Math.floor(ms / 1000))
}

export function formatRelativeTime(timestamp: number): string {
  if (!timestamp) return '刚刚'
  const now = Date.now()
  const diffSec = Math.floor((now - timestamp) / 1000)
  if (diffSec < 60) return '刚刚'
  if (diffSec < 3600) return `${Math.floor(diffSec / 60)}分钟前`
  if (diffSec < 86400) return `${Math.floor(diffSec / 3600)}小时前`
  if (diffSec < 86400 * 30) return `${Math.floor(diffSec / 86400)}天前`
  return dayjs(timestamp).format('YYYY-MM-DD')
}
