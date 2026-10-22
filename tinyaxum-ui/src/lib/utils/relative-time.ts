export function formatRelativeTime(
  input: string | Date,
  now: Date = new Date()
): string {
  const d = input instanceof Date ? input : new Date(input)
  if (Number.isNaN(d.getTime())) return ''

  const diffMs = now.getTime() - d.getTime()
  if (diffMs < 0) return toLocalDateStr(d)

  const sec = Math.floor(diffMs / 1000)
  if (sec < 10) return '刚刚'
  if (sec < 60) return `${sec}秒前`

  const min = Math.floor(sec / 60)
  if (min < 60) return `${min}分钟前`

  const hr = Math.floor(min / 60)
  if (hr < 24) return `${hr}小时前`

  const day = Math.floor(hr / 24)
  if (day < 7) return `${day}天前`

  return toLocalDateStr(d)
}

function toLocalDateStr(d: Date): string {
  const y = d.getFullYear()
  const m = String(d.getMonth() + 1).padStart(2, '0')
  const day = String(d.getDate()).padStart(2, '0')
  return `${y}-${m}-${day}`
}
