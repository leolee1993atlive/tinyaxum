<script lang="ts">
  import { Button } from '$lib/components/ui/button/index.js'
  import { Checkbox } from '$lib/components/ui/checkbox/index.js'
  import { Textarea } from '$lib/components/ui/textarea/index.js'
  import { Send } from '@lucide/svelte'

  interface Props {
    disabled: boolean
    currentSql: string
    initialContent?: string
    onSend: (payload: {
      content: string
      attachSchema: boolean
      currentSql?: string
    }) => void
  }

  let { disabled, currentSql, initialContent = '', onSend }: Props = $props()

  let input = $state('')
  let attachSchema = $state(false)
  let attachSql = $state(true)

  $effect(() => {
    if (initialContent) {
      input = initialContent
    }
  })

  function send() {
    const text = input.trim()
    if (!text || disabled) return
    onSend({
      content: text,
      attachSchema,
      currentSql: attachSql ? currentSql : undefined,
    })
    input = ''
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey && !e.isComposing) {
      e.preventDefault()
      send()
    }
  }
</script>

<div class="border-t p-3">
  <div class="text-muted-foreground mb-2 flex items-center gap-4 text-xs">
    <label class="flex items-center gap-1.5">
      <Checkbox bind:checked={attachSql} />
      <span>附带当前 SQL</span>
    </label>
    <!-- TODO: 表结构全量发送 token 过多,待改为按需选取后再开放 -->
    <!-- <label class="flex items-center gap-1.5">
      <Checkbox bind:checked={attachSchema} />
      <span>附带表结构</span>
    </label> -->
  </div>
  <div class="flex items-end gap-2">
    <Textarea
      bind:value={input}
      onkeydown={onKey}
      rows={3}
      placeholder="输入消息... (Enter 发送,Shift+Enter 换行)"
      class="resize-none"
    />
    <Button onclick={send} disabled={disabled || !input.trim()}>
      <Send class="h-4 w-4" />
    </Button>
  </div>
  <p class="text-muted-foreground mt-1 text-[10px]">
    勾选项将随消息发送至外部 AI 服务
  </p>
</div>
