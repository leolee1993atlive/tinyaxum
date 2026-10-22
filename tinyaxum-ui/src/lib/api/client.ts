import { connectionStore } from '#lib/stores/connection.js'
import { toast } from 'svelte-sonner'
import type { ApiResponse, PaginationParams, LoginResponse } from './types'

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '/api'
const USE_MOCK_API = import.meta.env.VITE_USE_MOCK_API === 'true'

interface QueuedRequest {
  resolve: (value: unknown) => void
  reject: (reason: unknown) => void
  retryRequest: () => Promise<unknown>
}

let isRefreshing = false
let refreshQueue: QueuedRequest[] = []

async function redirectToLogin(): Promise<void> {
  if (typeof window !== 'undefined') {
    try {
      await fetch(`${API_BASE_URL}/logout`, {
        method: 'POST',
        credentials: 'include',
      })
    } catch {}
    document.cookie = 'auth_user=; Path=/; Max-Age=0'
    toast.error('Session expired. Please log in again.')
    window.location.href = '/?session=expired'
  }
}

async function performTokenRefresh(): Promise<void> {
  if (typeof window === 'undefined') {
    throw new Error('Cannot refresh token on server side')
  }

  const response = await fetch(`${API_BASE_URL}/refreshToken`, {
    method: 'POST',
    credentials: 'include',
    headers: { 'Content-Type': 'application/json' },
    body: '{}',
  })

  if (!response.ok) {
    throw new Error('Token refresh failed')
  }

  const data: ApiResponse<unknown> = await response.json()

  if (data.code !== 200) {
    throw new Error(data.message || 'Token refresh failed')
  }
}

function processQueue(error: Error | null): void {
  refreshQueue.forEach((queuedRequest) => {
    if (error) {
      queuedRequest.reject(error)
    } else {
      queuedRequest
        .retryRequest()
        .then(queuedRequest.resolve)
        .catch(queuedRequest.reject)
    }
  })
  refreshQueue = []
}

function tryParseApiResponse<T>(bodyText: string): ApiResponse<T> | null {
  const t = bodyText.trim()
  if (!t) return null
  try {
    return JSON.parse(t) as ApiResponse<T>
  } catch {
    return null
  }
}

function httpFailureMessage(status: number, bodyText: string): string {
  const trimmed = bodyText.trim()
  const asApi = tryParseApiResponse<unknown>(bodyText)
  if (asApi && typeof asApi.message === 'string' && asApi.message !== '') {
    return asApi.message
  }
  if (trimmed.startsWith('{')) {
    try {
      const j: { error?: string } = JSON.parse(trimmed)
      if (typeof j.error === 'string' && j.error !== '') return j.error
    } catch {
      /* ignore */
    }
  }
  if (status === 400 && trimmed.includes('deserialize query string')) {
    return '请求参数无法解析，请检查分页或筛选条件是否正确'
  }
  if (trimmed.length > 0 && trimmed.length < 800) {
    return trimmed
  }
  return `请求失败（HTTP ${status}）`
}

export class ApiClientNotifiedError extends Error {
  override readonly name = 'ApiClientNotifiedError'
  constructor(message: string) {
    super(message)
  }
}

export function isApiClientNotifiedError(
  e: unknown
): e is ApiClientNotifiedError {
  return e instanceof ApiClientNotifiedError
}

function endpointPath(endpoint: string): string {
  const i = endpoint.indexOf('?')
  return i === -1 ? endpoint : endpoint.slice(0, i)
}

// 这些端点的 401 不代表"token 过期",不能触发刷新+重试:
// - /login 的 401 是凭证错误,应作为普通错误抛给登录页显示
// - /refreshToken 自身的 401 由 performTokenRefresh 直接处理
// 否则会陷入 login(401)->refresh(200)->login(401) 无限循环。
function shouldSkipTokenRefresh(endpoint: string): boolean {
  const path = endpointPath(endpoint)
  return path === '/login' || path === '/refreshToken'
}

function shouldSkipGlobalApiErrorToast(endpoint: string): boolean {
  const path = endpointPath(endpoint)
  return path === '/health' || path.endsWith('/health') || path === '/login'
}

function notifyApiErrorToast(endpoint: string, message: string): void {
  if (typeof window === 'undefined') return
  if (shouldSkipGlobalApiErrorToast(endpoint)) return
  toast.error(message)
}

function throwAfterApiNotify(endpoint: string, message: string): never {
  notifyApiErrorToast(endpoint, message)
  if (shouldSkipGlobalApiErrorToast(endpoint)) {
    throw new Error(message)
  }
  throw new ApiClientNotifiedError(message)
}

function assertOkApiResponse<T>(
  endpoint: string,
  response: Response,
  bodyText: string,
  parsed: ApiResponse<T> | null
): ApiResponse<T> {
  if (!response.ok) {
    throwAfterApiNotify(endpoint, httpFailureMessage(response.status, bodyText))
  }
  if (!parsed) {
    throwAfterApiNotify(
      endpoint,
      bodyText.trim()
        ? httpFailureMessage(response.status, bodyText)
        : '服务器返回了空响应'
    )
  }
  if (parsed.code !== 200) {
    throwAfterApiNotify(
      endpoint,
      parsed.message || `业务错误（${parsed.code}）`
    )
  }
  return parsed
}

function toHeadersRecord(
  headers: RequestInit['headers']
): Record<string, string> {
  if (!headers) return {}
  if (headers instanceof Headers) {
    const headersRecord: Record<string, string> = {}
    headers.forEach((value, key) => {
      headersRecord[key] = value
    })
    return headersRecord
  }
  if (Array.isArray(headers)) {
    return Object.fromEntries(headers)
  }
  return { ...headers }
}

class ApiClient {
  private baseUrl: string
  private useMock: boolean
  private refreshAttemptCount = 0

  constructor(baseUrl: string, useMock: boolean) {
    this.baseUrl = baseUrl
    this.useMock = useMock
  }

  getBaseUrl(): string {
    return this.baseUrl
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    if (this.useMock) {
      console.warn(
        `[Mock API Enabled] ${options.method || 'GET'} ${endpoint} - Set VITE_USE_MOCK_API=false to use real API`
      )
      return {
        code: 200,
        message: 'Success',
        data: {} as T,
      }
    }

    const url = `${this.baseUrl}${endpoint}`

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...toHeadersRecord(options.headers),
    }

    try {
      const response = await fetch(url, {
        ...options,
        headers,
        credentials: 'include',
      })

      connectionStore.markConnected()

      if (response.status === 401 && !shouldSkipTokenRefresh(endpoint)) {
        return this.handle401<T>(endpoint, options)
      }

      this.refreshAttemptCount = 0

      const bodyText = await response.text()
      const parsed = tryParseApiResponse<T>(bodyText)
      return assertOkApiResponse(endpoint, response, bodyText, parsed)
    } catch (error) {
      if (isApiClientNotifiedError(error)) {
        throw error
      }
      if (error instanceof TypeError && error.message.includes('fetch')) {
        connectionStore.markDisconnected()
        throwAfterApiNotify(
          endpoint,
          '无法连接到服务器，请检查后端服务是否运行'
        )
      }

      if (error instanceof Error) {
        if (
          error.message.includes('NetworkError') ||
          error.message.includes('ECONNREFUSED') ||
          error.message.includes('Failed to fetch')
        ) {
          connectionStore.markDisconnected()
          throwAfterApiNotify(
            endpoint,
            '无法连接到服务器，请检查后端服务是否运行'
          )
        }
      }

      throw error
    }
  }

  private async handle401<T>(
    endpoint: string,
    options: RequestInit
  ): Promise<ApiResponse<T>> {
    if (this.refreshAttemptCount >= 3) {
      this.refreshAttemptCount = 0
      processQueue(new Error('Max refresh retries exceeded'))
      await redirectToLogin()
      throw new Error('Max refresh retries exceeded')
    }

    if (isRefreshing) {
      return new Promise<ApiResponse<T>>((resolve, reject) => {
        refreshQueue.push({
          resolve: resolve as (value: unknown) => void,
          reject: reject as (reason: unknown) => void,
          retryRequest: () => this.request<T>(endpoint, options),
        })
      })
    }

    isRefreshing = true
    this.refreshAttemptCount++

    try {
      await performTokenRefresh()

      // 不在此处重置 refreshAttemptCount:刷新成功 ≠ 实际请求成功。
      // 若重试后仍是 401,需让计数累加直到触发上面的 >=3 熔断,
      // 否则刷新一直成功、请求一直 401 会无限循环。
      // 计数只在真正拿到非 401 响应时重置(见 request() 中 markConnected 之后)。
      processQueue(null)

      return this.request<T>(endpoint, options)
    } catch (error) {
      const err =
        error instanceof Error ? error : new Error('Token refresh failed')

      processQueue(err)

      await redirectToLogin()

      throw err
    } finally {
      isRefreshing = false
    }
  }

  async get<T>(
    endpoint: string,
    params?: Record<string, string>
  ): Promise<ApiResponse<T>> {
    const queryString = params ? `?${new URLSearchParams(params)}` : ''
    return this.request<T>(`${endpoint}${queryString}`, { method: 'GET' })
  }

  async post<T>(endpoint: string, data: unknown): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'POST',
      body: JSON.stringify(data),
    })
  }

  async put<T>(endpoint: string, data: unknown): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'PUT',
      body: JSON.stringify(data),
    })
  }

  async delete<T>(endpoint: string): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, { method: 'DELETE' })
  }
}

export const apiClient = new ApiClient(API_BASE_URL, USE_MOCK_API)
