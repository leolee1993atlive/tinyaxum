import { getContext, setContext } from 'svelte'
import { authApi } from '#lib/api/auth.js'
import { toast } from 'svelte-sonner'

export interface User {
  id: number
  username: string
}

interface JwtPayload {
  id?: number | string
  username?: string
  exp?: number
}

function decodeJwt(token: string): JwtPayload | null {
  try {
    const base64Payload = token.split('.')[1]
    if (!base64Payload) return null
    const base64 = base64Payload.replace(/-/g, '+').replace(/_/g, '/')
    const padded = base64 + '='.repeat((4 - (base64.length % 4)) % 4)
    return JSON.parse(atob(padded)) as JwtPayload
  } catch {
    return null
  }
}

class AuthState {
  user = $state<User | null>(null)
  isLoading = $state(false)
  error = $state<string | null>(null)

  isAuthenticated = $derived(this.user !== null)

  constructor() {
    this.checkAuth()
  }

  private setAuthCookie(userId: number, username: string): void {
    if (typeof window === 'undefined') return
    const payload = btoa(JSON.stringify({ id: userId, username }))
    document.cookie = `auth_user=${payload}; Path=/; SameSite=Strict; Max-Age=18000`
  }

  private clearAuthCookie(): void {
    if (typeof window === 'undefined') return
    document.cookie = 'auth_user=; Path=/; Max-Age=0'
  }

  login = async (username: string, password: string): Promise<boolean> => {
    this.isLoading = true
    this.error = null

    try {
      const response = await authApi.login({ username, password })
      const loginData = response.data

      if (loginData) {
        const payload = decodeJwt(loginData.accessToken)
        const userId = payload?.id ? parseInt(String(payload.id), 10) : 0
        this.user = { id: userId, username }
        this.setAuthCookie(userId, username)
        return true
      }

      return false
    } catch (err) {
      this.error = err instanceof Error ? err.message : 'Login failed'
      return false
    } finally {
      this.isLoading = false
    }
  }

  logout = async (): Promise<void> => {
    try {
      await fetch('/api/logout', { method: 'POST', credentials: 'include' })
    } catch {}
    this.clearAuthCookie()
    this.user = null
    this.error = null
    toast.success('登出成功')
  }

  checkAuth = (): boolean => {
    if (typeof window === 'undefined') return false
    const match = document.cookie.match(/(?:^|;\s*)auth_user=([^;]*)/)
    if (match) {
      try {
        const info = JSON.parse(atob(match[1]))
        this.user = { id: info.id, username: info.username }
        return true
      } catch {
        this.clearAuthCookie()
        this.user = null
        return false
      }
    }
    this.user = null
    return false
  }
}

const AUTH_CONTEXT_KEY = Symbol.for('scn-auth')

export function setAuthContext() {
  const authState = new AuthState()
  return setContext(AUTH_CONTEXT_KEY, authState)
}

export function useAuth() {
  const context = getContext<AuthState>(AUTH_CONTEXT_KEY)
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider')
  }
  return context
}
