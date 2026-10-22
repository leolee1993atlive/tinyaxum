// API Response Types - matches backend R<T> structure
export interface ApiResponse<T> {
  code: number
  message: string
  data?: T
}

export interface ApiError {
  code: string
  message: string
  details?: Record<string, string[]>
}

// Pagination Types
export interface PaginationParams {
  pageNo: number
  pageSize: number
  sort?: string
  order?: 'asc' | 'desc'
}

/** Matches backend `PageResult` (common/vo/page.rs), camelCase. */
export interface PageResult<T> {
  records: T[]
  total: number
  pages: number
  current: number
  size: number
}

export type UserPageParams = PaginationParams & {
  keyword?: string
  username?: string
  nickName?: string
  active?: string
}

/**
 * Serialize pagination + optional filters for GET query string.
 * Omits undefined/null and empty-string optional filters.
 */
export function toPageQueryRecord(
  base: PaginationParams,
  extra?: Record<string, string | number | undefined | null>
): Record<string, string> {
  const r: Record<string, string> = {
    pageNo: String(base.pageNo),
    pageSize: String(base.pageSize),
  }
  if (base.sort) r.sort = base.sort
  if (base.order) r.order = base.order
  if (!extra) return r
  for (const [k, v] of Object.entries(extra)) {
    if (v === undefined || v === null) continue
    if (typeof v === 'string' && v.trim() === '') continue
    r[k] = String(v)
  }
  return r
}

// User API Types - matches backend UserCreateBo, UserUpdateBo
export interface UserCreateRequest {
  username: string
  password: string
  nickName: string
  email?: string
  active: string
}

export interface UserUpdateRequest {
  userId: number
  username: string
  nickName: string
  email?: string
  active: string
}

// Auth API Types - matches backend AuthBody and AuthPayload
export interface LoginRequest {
  username: string
  password: string
}

export interface LoginResponse {
  accessToken: string
  refreshToken: string
  tokenType: string
}
