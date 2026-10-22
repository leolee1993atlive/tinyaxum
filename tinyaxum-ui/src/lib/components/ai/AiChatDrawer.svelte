<script lang="ts">
  import * as Sheet from '$lib/components/ui/sheet/index.js'
  import AiComposer from './AiComposer.svelte'
  import AiConversationTabs from './AiConversationTabs.svelte'
  import AiMessageList from './AiMessageList.svelte'
  import ApplyToSnippetDialog from './ApplyToSnippetDialog.svelte'
  import {
    createAiChatState,
    type AiChatState,
  } from './ai-chat-state.svelte.ts'

  interface Props {
    open: boolean
    sqlQueryId: number
    currentSql: string
    initialInput?: string
    /** 当前 SQL 中引用的片段名（供「应用到片段」置顶/预选） */
    usedSnippetNames?: string[]
    /** 片段保存成功后回调（父组件刷新片段映射与侧栏） */
    onSnippetUpdated?: () => void
    onOpenChange: (open: boolean) => void
  }

  let {
    open,
    sqlQueryId,
    currentSql,
    initialInput = '',
    usedSnippetNames = [],
    onSnippetUpdated,
    onOpenChange,
  }: Props = $props()

  // 注意：变量不要命名为 state —— 模板引用会触发 svelte2tsx 生成
  // `$state` store 订阅标识符，与 $state rune 冲突导致类型报错
  let chat = $state<AiChatState | null>(null)
  // 待应用到片段的 AI 代码块内容；null 表示弹窗关闭
  let applyCode = $state<string | null>(null)

  $effect(() => {
    if (open) {
      const s = createAiChatState(sqlQueryId)
      chat = s
      void s.loadAll()
    } else {
      chat = null
    }
  })

  function handleApplyOpenChange(o: boolean) {
    if (!o) applyCode = null
  }
</script>

<Sheet.Root {open} {onOpenChange}>
  <Sheet.Content side="right" class="flex w-full flex-col p-0 sm:max-w-lg">
    <Sheet.Header class="border-b px-3 py-2">
      <Sheet.Title>AI 助手</Sheet.Title>
    </Sheet.Header>
    {#if chat}
      <AiConversationTabs
        conversations={chat.conversations}
        currentConvId={chat.currentConvId}
        providers={chat.providers}
        onSelect={(id) => chat!.selectConversation(id)}
        onNew={() => chat!.newConversation()}
        onDelete={() => chat!.deleteCurrentConversation()}
        onSwitchProvider={(id) => chat!.switchProvider(id)}
      />
      <AiMessageList
        messages={chat.messages}
        streamingContent={chat.streamingContent}
        isStreaming={chat.isStreaming}
        onApplyCode={(code) => (applyCode = code)}
      />
      <AiComposer
        disabled={chat.isStreaming || chat.currentConvId == null}
        {currentSql}
        initialContent={initialInput}
        onSend={(payload) => chat!.sendMessage(payload)}
      />
    {:else}
      <div class="text-muted-foreground p-4 text-sm">加载中...</div>
    {/if}

    <!-- Dialog 必须嵌套在 Sheet.Content 内部，保证弹层层叠在 AI 抽屉之上
         （与 SqlQueryEditorDrawer 中预览 Dialog 的处理一致） -->
    {#if applyCode !== null}
      <ApplyToSnippetDialog
        open={applyCode !== null}
        code={applyCode}
        {usedSnippetNames}
        onOpenChange={handleApplyOpenChange}
        onApplied={onSnippetUpdated}
      />
    {/if}
  </Sheet.Content>
</Sheet.Root>
