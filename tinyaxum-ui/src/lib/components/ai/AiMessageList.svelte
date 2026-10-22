<script lang="ts">
  import { tick } from 'svelte'
  import type { AiMessageVo } from '$lib/api/types'
  import AiMessageBubble from './AiMessageBubble.svelte'

  interface Props {
    messages: AiMessageVo[]
    streamingContent: string
    isStreaming: boolean
    onApplyCode?: (code: string) => void
  }

  let { messages, streamingContent, isStreaming, onApplyCode }: Props = $props()
  let scroller: HTMLDivElement | null = $state(null)

  $effect(() => {
    void messages
    void streamingContent
    tick().then(() => {
      if (scroller) scroller.scrollTop = scroller.scrollHeight
    })
  })
</script>

<div bind:this={scroller} class="flex-1 space-y-3 overflow-y-auto px-3 py-3">
  {#each messages as m (m.aiMessageId)}
    <AiMessageBubble
      role={m.role}
      content={m.content}
      onApplyCode={m.role === 'assistant' ? onApplyCode : undefined}
    />
  {/each}
  {#if isStreaming && streamingContent}
    <AiMessageBubble
      role="assistant"
      content={streamingContent}
      {onApplyCode}
    />
  {:else if isStreaming}
    <div class="text-muted-foreground text-xs">AI 正在思考...</div>
  {/if}
</div>
