import { writable, derived } from 'svelte/store'

export type ConnectionStatus = 'connected' | 'disconnected' | 'checking'

interface ConnectionState {
  status: ConnectionStatus
  lastCheck: number
  retryCount: number
}

function createConnectionStore() {
  const { subscribe, set, update } = writable<ConnectionState>({
    status: 'connected',
    lastCheck: Date.now(),
    retryCount: 0,
  })

  let checkInterval: ReturnType<typeof setInterval> | null = null
  let isChecking = false

  const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '/api'

  async function checkConnection(): Promise<boolean> {
    if (isChecking) return false
    isChecking = true

    try {
      // 使用一个简单的健康检查端点，或者使用现有的登录端点
      // 这里使用一个轻量级的请求来检测后端是否可用
      const controller = new AbortController()
      const timeoutId = setTimeout(() => controller.abort(), 5000)

      const response = await fetch(`${API_BASE_URL}/health`, {
        method: 'GET',
        signal: controller.signal,
      })

      clearTimeout(timeoutId)

      if (
        response &&
        (response.ok || response.status === 405 || response.status === 401)
      ) {
        // 405 = Method Not Allowed (OPTIONS 不被支持但服务正常)
        // 401 = Unauthorized (服务正常但需认证)
        update((state) => ({
          ...state,
          status: 'connected',
          lastCheck: Date.now(),
          retryCount: 0,
        }))
        return true
      }

      throw new Error('Connection failed')
    } catch {
      update((state) => ({
        ...state,
        status: 'disconnected',
        lastCheck: Date.now(),
        retryCount: state.retryCount + 1,
      }))
      return false
    } finally {
      isChecking = false
    }
  }

  function startChecking(intervalMs = 5000) {
    if (checkInterval) return

    checkInterval = setInterval(() => {
      checkConnection()
    }, intervalMs)
  }

  function stopChecking() {
    if (checkInterval) {
      clearInterval(checkInterval)
      checkInterval = null
    }
  }

  function markDisconnected() {
    update((state) => ({
      ...state,
      status: 'disconnected',
      lastCheck: Date.now(),
      retryCount: state.retryCount + 1,
    }))
  }

  function markConnected() {
    update((state) => ({
      ...state,
      status: 'connected',
      lastCheck: Date.now(),
      retryCount: 0,
    }))
  }

  return {
    subscribe,
    checkConnection,
    startChecking,
    stopChecking,
    markDisconnected,
    markConnected,
  }
}

export const connectionStore = createConnectionStore()

// Derived store for easy access to connection status
export const isConnected = derived(
  connectionStore,
  ($state) => $state.status === 'connected'
)
