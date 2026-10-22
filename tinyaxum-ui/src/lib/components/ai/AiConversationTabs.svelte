<script lang="ts">
  import type { AiConversationListVo, AiProviderListVo } from '$lib/api/types'
  import { Button } from '$lib/components/ui/button/index.js'
  import * as Select from '$lib/components/ui/select/index.js'
  import { Plus, Trash2 } from '@lucide/svelte'

  interface Props {
    conversations: AiConversationListVo[]
    currentConvId: number | null
    providers: AiProviderListVo[]
    onSelect: (id: number) => void
    onNew: () => void
    onDelete: () => void
    onSwitchProvider: (providerId: number) => void
  }

  let {
    conversations,
    currentConvId,
    providers,
    onSelect,
    onNew,
    onDelete,
    onSwitchProvider,
  }: Props = $props()

  const currentConv = $derived(
    conversations.find((c) => c.aiConversationId === currentConvId) ?? null
  )
  const currentProviderId = $derived(currentConv?.aiProviderId ?? null)
</script>

<div class="flex items-center gap-2 border-b px-3 py-2">
  <div class="flex flex-1 items-center gap-1 overflow-x-auto">
    {#each conversations as c (c.aiConversationId)}
      <button
        type="button"
        class={c.aiConversationId === currentConvId
          ? 'bg-primary text-primary-foreground rounded px-2 py-1 text-xs'
          : 'bg-muted hover:bg-muted/70 rounded px-2 py-1 text-xs'}
        onclick={() => onSelect(c.aiConversationId)}
      >
        {c.title}
      </button>
    {/each}
    <Button size="sm" variant="ghost" onclick={onNew} aria-label="新会话">
      <Plus class="h-4 w-4" />
    </Button>
  </div>

  <Select.Root
    type="single"
    value={currentProviderId == null ? undefined : String(currentProviderId)}
    onValueChange={(v: string) => v && onSwitchProvider(Number(v))}
  >
    <Select.Trigger class="h-7 w-[140px] text-xs">
      {providers.find((p) => p.aiProviderId === currentProviderId)?.name ??
        '选择模型'}
    </Select.Trigger>
    <Select.Content>
      {#each providers as p (p.aiProviderId)}
        <Select.Item value={String(p.aiProviderId)} label={p.name}>
          {p.name}
        </Select.Item>
      {/each}
    </Select.Content>
  </Select.Root>

  <Button size="sm" variant="ghost" onclick={onDelete} aria-label="删除会话">
    <Trash2 class="h-4 w-4" />
  </Button>
</div>
