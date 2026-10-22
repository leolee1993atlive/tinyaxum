import type {
  TransTask,
  TransTaskBatchCreateRequest,
  TransTaskCreate,
  TransTaskUpdate,
} from '../../routes/(app)/tasks/data/schema.js'
import { apiClient } from './client.js'
import type { PageResult, TransTaskPageParams } from './types.js'
import { toPageQueryRecord } from './types.js'

export type SyncProgressEvent =
  | { type: 'exporting'; phase: 'full' | 'delta' | 'initial' }
  | { type: 'progress'; rowsDone: number }
  | { type: 'done'; rowCount: number; newWatermark: string | null }
  | { type: 'error'; message: string }

export const transtaskApi = {
  async list(): Promise<TransTask[]> {
    const response = await apiClient.get<TransTask[]>('/transtask')
    return response.data ?? []
  },

  page: (params: TransTaskPageParams) => {
    const { pageNo, pageSize, sort, order, ...filters } = params
    const q = toPageQueryRecord(
      { pageNo, pageSize, sort, order },
      {
        keyword: filters.keyword,
        dataSourceId: filters.dataSourceId,
        tableName: filters.tableName,
        tableComment: filters.tableComment,
        taskStatus: filters.taskStatus,
      }
    )
    return apiClient.get<PageResult<TransTask>>('/transtask/page', q)
  },

  async getById(id: number): Promise<TransTask | null> {
    const response = await apiClient.get<TransTask>(`/transtask/${id}`)
    return response.data ?? null
  },

  async create(data: TransTaskCreate): Promise<boolean> {
    const response = await apiClient.post<TransTask>('/transtask', data)
    return response.code === 200
  },

  async batchCreate(data: TransTaskBatchCreateRequest): Promise<void> {
    await apiClient.post<unknown>('/transtask/batch', data)
  },

  async update(data: TransTaskUpdate): Promise<boolean> {
    const response = await apiClient.put<TransTask>('/transtask', data)
    return response.code === 200
  },

  async delete(id: number): Promise<boolean> {
    const response = await apiClient.delete<void>(`/transtask/${id}`)
    return response.code === 200
  },

  async trueDelete(id: number): Promise<boolean> {
    const response = await apiClient.delete<void>(`/transtask/delete/${id}`)
    return response.code === 200
  },

  async test(id: number): Promise<boolean> {
    const response = await apiClient.post<void>(`/transtask/test/${id}`, null)
    return response.code === 200
  },

  async *runWithProgress(
    id: number,
    signal?: AbortSignal
  ): AsyncGenerator<SyncProgressEvent> {
    const response = await fetch(
      `${apiClient.getBaseUrl()}/transtask/run/${id}`,
      {
        method: 'POST',
        credentials: 'include',
        signal,
      }
    )

    if (!response.ok || !response.body) {
      throw new Error(`HTTP ${response.status}`)
    }

    const reader = response.body.getReader()
    const decoder = new TextDecoder()
    let buffer = ''

    signal?.addEventListener('abort', () => reader.cancel(), { once: true })

    while (true) {
      const { done, value } = await reader.read()
      if (done) break

      buffer += decoder.decode(value, { stream: true })
      const parts = buffer.split('\n\n')
      buffer = parts.pop() ?? ''

      for (const block of parts) {
        for (const line of block.split('\n')) {
          if (line.startsWith('data: ')) {
            try {
              yield JSON.parse(line.slice(6)) as SyncProgressEvent
            } catch {
              // ignore malformed lines
            }
          }
        }
      }
    }
  },

  async start(id: number): Promise<boolean> {
    const response = await apiClient.post<void>(`/transtask/start/${id}`, null)
    return response.code === 200
  },

  async stop(id: number): Promise<boolean> {
    const response = await apiClient.post<void>(`/transtask/stop/${id}`, null)
    return response.code === 200
  },

  subscribeToEvents(
    onEvent: (event: { taskId: number; rowCount: number }) => void,
    onError?: (err: Error) => void
  ): () => void {
    const controller = new AbortController()
    const { signal } = controller

    ;(async () => {
      try {
        const response = await fetch(
          `${apiClient.getBaseUrl()}/transtask/events`,
          {
            credentials: 'include',
            signal,
          }
        )
        if (!response.ok || !response.body) return

        const reader = response.body.getReader()
        const decoder = new TextDecoder()
        let buffer = ''

        signal.addEventListener('abort', () => reader.cancel(), { once: true })

        while (true) {
          const { done, value } = await reader.read()
          if (done) break
          buffer += decoder.decode(value, { stream: true })
          const parts = buffer.split('\n\n')
          buffer = parts.pop() ?? ''
          for (const block of parts) {
            for (const line of block.split('\n')) {
              if (line.startsWith('data: ')) {
                try {
                  const eventData = JSON.parse(line.slice(6))
                  onEvent(eventData)
                } catch {
                  // ignore malformed
                }
              }
            }
          }
        }
      } catch (err) {
        if (err instanceof DOMException && err.name === 'AbortError') return
        onError?.(err instanceof Error ? err : new Error(String(err)))
      }
    })()

    return () => controller.abort()
  },
}
