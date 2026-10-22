import { apiClient } from './client'
import type {
  AiConversationCreateRequest,
  AiConversationListVo,
  AiConversationUpdateRequest,
  AiMessageVo,
  AiSseEvent,
  SendMessageRequest,
} from './types'

export const aiChatApi = {
  listConversations: (sqlQueryId: number) =>
    apiClient.get<AiConversationListVo[]>('/ai/conversations', {
      sqlQueryId: String(sqlQueryId),
    }),
  createConversation: (data: AiConversationCreateRequest) =>
    apiClient.post<number>('/ai/conversations', data),
  updateConversation: (id: number, data: AiConversationUpdateRequest) =>
    apiClient.put<void>(`/ai/conversations/${id}`, data),
  deleteConversation: (id: number) =>
    apiClient.delete<void>(`/ai/conversations/${id}`),
  listMessages: (convId: number) =>
    apiClient.get<AiMessageVo[]>(`/ai/conversations/${convId}/messages`),
}

export async function* sendMessageStream(
  conversationId: number,
  body: SendMessageRequest
): AsyncGenerator<AiSseEvent> {
  const res = await fetch(
    `${apiClient.getBaseUrl()}/ai/conversations/${conversationId}/messages`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Accept: 'text/event-stream',
      },
      credentials: 'include',
      body: JSON.stringify(body),
    }
  )
  if (!res.ok || !res.body) {
    yield {
      type: 'error',
      code: res.status === 401 ? 'SESSION_EXPIRED' : 'HTTP_ERROR',
      message:
        res.status === 401
          ? '会话已过期,请刷新页面后重试'
          : `${res.status} ${res.statusText}`,
    }
    return
  }
  const reader = res.body.getReader()
  const decoder = new TextDecoder('utf-8')
  let buf = ''
  let currentEvent = ''
  let currentData = ''

  function dispatch(): AiSseEvent | null {
    if (!currentEvent || !currentData) return null
    try {
      const data = JSON.parse(currentData)
      switch (currentEvent) {
        case 'user_saved':
          return { type: 'user_saved', id: data.id }
        case 'delta':
          return { type: 'delta', content: data.content }
        case 'done':
          return {
            type: 'done',
            id: data.id,
            promptTokens: data.promptTokens ?? null,
            completionTokens: data.completionTokens ?? null,
          }
        case 'error':
          return {
            type: 'error',
            code: data.code ?? 'UNKNOWN',
            message: data.message ?? '',
          }
      }
    } catch {
      // ignore malformed chunk
    }
    return null
  }

  while (true) {
    const { done, value } = await reader.read()
    if (done) break
    buf += decoder.decode(value, { stream: true })
    let idx: number
    while ((idx = buf.indexOf('\n\n')) >= 0) {
      const block = buf.slice(0, idx)
      buf = buf.slice(idx + 2)
      currentEvent = ''
      currentData = ''
      for (const line of block.split('\n')) {
        if (line.startsWith('event:')) {
          currentEvent = line.slice(6).trim()
        } else if (line.startsWith('data:')) {
          currentData += line.slice(5).trim()
        }
      }
      const ev = dispatch()
      if (ev) yield ev
      if (ev?.type === 'done' || ev?.type === 'error') return
    }
  }
}
