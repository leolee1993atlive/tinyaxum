import { aiChatApi, sendMessageStream } from '$lib/api/ai-chat'
import { aiProvidersApi } from '$lib/api/ai-providers'
import type {
  AiConversationListVo,
  AiMessageVo,
  AiProviderListVo,
} from '$lib/api/types'
import { toast } from 'svelte-sonner'

export interface AiChatState {
  readonly providers: AiProviderListVo[]
  readonly conversations: AiConversationListVo[]
  readonly currentConvId: number | null
  readonly messages: AiMessageVo[]
  readonly streamingContent: string
  readonly isStreaming: boolean
  loadAll(): Promise<void>
  selectConversation(id: number): Promise<void>
  newConversation(): Promise<void>
  switchProvider(providerId: number): Promise<void>
  deleteCurrentConversation(): Promise<void>
  sendMessage(input: {
    content: string
    attachSchema: boolean
    currentSql?: string
  }): Promise<void>
}

export function createAiChatState(sqlQueryId: number): AiChatState {
  let providers = $state<AiProviderListVo[]>([])
  let conversations = $state<AiConversationListVo[]>([])
  let currentConvId = $state<number | null>(null)
  let messages = $state<AiMessageVo[]>([])
  let streamingContent = $state('')
  let isStreaming = $state(false)

  async function loadProviders() {
    try {
      const res = await aiProvidersApi.list()
      providers = res.data ?? []
    } catch (e) {
      toast.error(`加载 provider 失败: ${e}`)
    }
  }

  async function loadConversations() {
    try {
      const res = await aiChatApi.listConversations(sqlQueryId)
      conversations = res.data ?? []
    } catch (e) {
      toast.error(`加载会话失败: ${e}`)
    }
  }

  async function loadMessages(convId: number) {
    try {
      const res = await aiChatApi.listMessages(convId)
      messages = res.data ?? []
    } catch (e) {
      toast.error(`加载消息失败: ${e}`)
    }
  }

  async function loadAll() {
    await Promise.all([loadProviders(), loadConversations()])
    if (conversations.length === 0) {
      await newConversation()
    } else {
      await selectConversation(conversations[0].aiConversationId)
    }
  }

  async function selectConversation(id: number) {
    currentConvId = id
    streamingContent = ''
    await loadMessages(id)
  }

  async function newConversation() {
    try {
      const res = await aiChatApi.createConversation({ sqlQueryId })
      const newId = res.data!
      await loadConversations()
      await selectConversation(newId)
    } catch (e) {
      toast.error(`新建会话失败: ${e}`)
    }
  }

  async function switchProvider(providerId: number) {
    if (currentConvId == null) return
    try {
      await aiChatApi.updateConversation(currentConvId, {
        aiProviderId: providerId,
      })
      await loadConversations()
    } catch (e) {
      toast.error(`切换模型失败: ${e}`)
    }
  }

  async function deleteCurrentConversation() {
    if (currentConvId == null) return
    if (!confirm('删除当前会话?')) return
    try {
      await aiChatApi.deleteConversation(currentConvId)
      currentConvId = null
      messages = []
      await loadConversations()
      if (conversations.length === 0) {
        await newConversation()
      } else {
        await selectConversation(conversations[0].aiConversationId)
      }
    } catch (e) {
      toast.error(`删除失败: ${e}`)
    }
  }

  async function sendMessage(input: {
    content: string
    attachSchema: boolean
    currentSql?: string
  }) {
    if (currentConvId == null) {
      toast.error('请先选择会话')
      return
    }
    if (isStreaming) return
    isStreaming = true
    streamingContent = ''
    const optimisticUser: AiMessageVo = {
      aiMessageId: -1,
      aiConversationId: currentConvId,
      role: 'user',
      content: input.content,
      attachedSchema: input.attachSchema ? 1 : 0,
      promptTokens: null,
      completionTokens: null,
      version: 1,
      createdBy: 0,
      createdDate: new Date().toISOString(),
      lastModifiedBy: 0,
      lastModifiedDate: new Date().toISOString(),
      deletedBy: null,
      deletedDate: null,
    }
    messages = [...messages, optimisticUser]

    try {
      for await (const ev of sendMessageStream(currentConvId, input)) {
        if (ev.type === 'user_saved') {
          const last = messages[messages.length - 1]
          if (last && last.aiMessageId === -1) {
            messages = [
              ...messages.slice(0, -1),
              { ...last, aiMessageId: ev.id },
            ]
          }
        } else if (ev.type === 'delta') {
          streamingContent += ev.content
        } else if (ev.type === 'done') {
          messages = [
            ...messages,
            {
              aiMessageId: ev.id,
              aiConversationId: currentConvId,
              role: 'assistant',
              content: streamingContent,
              attachedSchema: 0,
              promptTokens: ev.promptTokens,
              completionTokens: ev.completionTokens,
              version: 1,
              createdBy: 0,
              createdDate: new Date().toISOString(),
              lastModifiedBy: 0,
              lastModifiedDate: new Date().toISOString(),
              deletedBy: null,
              deletedDate: null,
            },
          ]
          streamingContent = ''
        } else if (ev.type === 'error') {
          if (ev.code === 'PROVIDER_DELETED') {
            toast.error('当前模型已被删除,请在抽屉顶部切换模型后重试')
          } else {
            toast.error(`${ev.code}: ${ev.message}`)
          }
        }
      }
    } finally {
      isStreaming = false
    }
  }

  return {
    get providers() {
      return providers
    },
    get conversations() {
      return conversations
    },
    get currentConvId() {
      return currentConvId
    },
    get messages() {
      return messages
    },
    get streamingContent() {
      return streamingContent
    },
    get isStreaming() {
      return isStreaming
    },
    loadAll,
    selectConversation,
    newConversation,
    switchProvider,
    deleteCurrentConversation,
    sendMessage,
  }
}
