<script lang="ts">
  import type { Snippet } from 'svelte'
  import * as Collapsible from '$lib/components/ui/collapsible/index.js'
  import { cn } from '$lib/utils.js'
  import { ChevronRight } from '@lucide/svelte'

  let {
    title,
    count,
    level = 1,
    open = false,
    onexpand = undefined,
    onopenchange = undefined,
    children,
  }: {
    title: string
    count?: number
    level?: 1 | 2
    open?: boolean
    onexpand?: () => void
    onopenchange?: (open: boolean) => void
    children: Snippet
  } = $props()

  function handleOpenChange(next: boolean) {
    onopenchange?.(next)
    if (next) onexpand?.()
  }
</script>

<Collapsible.Root
  data-slot="dashboard-group"
  {open}
  onOpenChange={handleOpenChange}
>
  <Collapsible.Trigger
    class={cn(
      'hover:bg-muted/50 flex w-full items-center gap-2 rounded-md px-2 py-2 text-left',
      level === 1 ? 'text-base font-semibold' : 'text-sm font-medium'
    )}
  >
    <ChevronRight
      class={cn('size-4 shrink-0 transition-transform', open && 'rotate-90')}
    />
    <span class="flex-1 truncate">{title}</span>
    {#if count !== undefined}
      <span class="text-muted-foreground text-xs tabular-nums">{count}</span>
    {/if}
  </Collapsible.Trigger>
  <Collapsible.Content class={cn('pt-2', level === 1 ? 'pl-2' : 'pl-4')}>
    {@render children()}
  </Collapsible.Content>
</Collapsible.Root>
