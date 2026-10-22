/**
 * 可拖拽分割条的百分比夹取与 localStorage 持久化（纯函数，不依赖组件）。
 * 语义：非法输入一律退回 fallback；越界数字夹到 [SPLIT_MIN, SPLIT_MAX]。
 */
export const SPLIT_MIN = 20
export const SPLIT_MAX = 80
export const SPLIT_DEFAULT = 50

function clampRange(n: number): number {
  return Math.min(SPLIT_MAX, Math.max(SPLIT_MIN, n))
}

/** 夹到 [SPLIT_MIN, SPLIT_MAX]；非有限数返回 fallback（fallback 本身也会被夹）。 */
export function clampSplit(pct: number, fallback = SPLIT_DEFAULT): number {
  const safeFallback = Number.isFinite(fallback)
    ? clampRange(fallback)
    : SPLIT_DEFAULT
  return Number.isFinite(pct) ? clampRange(pct) : safeFallback
}

/** 读 localStorage：无 window / 无值 / 非法字符串 → fallback；越界数字夹到边界。 */
export function loadSplit(key: string, fallback = SPLIT_DEFAULT): number {
  if (typeof window === 'undefined') return clampSplit(fallback)
  try {
    const raw = window.localStorage.getItem(key)
    if (raw === null || raw.trim() === '') return clampSplit(fallback)
    return clampSplit(Number(raw), fallback)
  } catch {
    return clampSplit(fallback)
  }
}

/** 写 localStorage（夹后再写）；无 window 或写入失败（隐私模式/配额）静默。 */
export function saveSplit(key: string, pct: number): void {
  if (typeof window === 'undefined') return
  try {
    window.localStorage.setItem(key, String(clampSplit(pct)))
  } catch {
    // 隐私模式 / 配额满：忽略，比例仅本次会话有效
  }
}
