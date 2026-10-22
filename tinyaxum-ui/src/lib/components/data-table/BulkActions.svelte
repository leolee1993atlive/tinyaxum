<script lang="ts" generics="TData extends RowData">
  import { Badge } from '#lib/components/ui/badge/index.js'
  import { Button } from '#lib/components/ui/button/index.js'
  import { Separator } from '#lib/components/ui/separator/index.js'
  import * as Tooltip from '#lib/components/ui/tooltip/index.js'
  import { X } from '@lucide/svelte'
  import type { RowData } from '@tanstack/table-core'
  import type { AppTable } from '#lib/table-features.js'

  let {
    table,
    entityName,
    children,
  }: {
    table: AppTable<TData>
    entityName: string
    children: import('svelte').Snippet
  } = $props()

  const selectedRows = $derived(table.getFilteredSelectedRowModel().rows)
  const selectedCount = $derived(selectedRows.length)

  function handleClearSelection() {
    table.resetRowSelection()
  }
</script>

{#if selectedCount > 0}
  <div
    class="bg-background animate-in fade-in slide-in-from-bottom-4 fixed bottom-4 left-1/2 z-50 flex -translate-x-1/2 items-center gap-2 rounded-full border px-4 py-2 shadow-lg transition-all duration-300 sm:px-6"
    role="toolbar"
    aria-label="Bulk actions"
  >
    <div class="flex items-center gap-2">
      <Badge
        variant="secondary"
        class="h-6 rounded-full px-2 font-mono text-xs"
      >
        {selectedCount}
      </Badge>
      <span class="text-muted-foreground hidden text-sm sm:inline">
        {entityName}{selectedCount > 1 ? 's' : ''} selected
      </span>
    </div>

    <Separator orientation="vertical" class="mx-1 h-8" />

    <div class="flex items-center gap-1 sm:gap-2">
      {@render children()}
    </div>

    <Separator orientation="vertical" class="mx-1 h-8" />

    <Tooltip.Root>
      <Tooltip.Trigger>
        {#snippet child({ props })}
          <Button
            variant="ghost"
            size="icon"
            class="size-8 rounded-full"
            onclick={handleClearSelection}
            {...props}
          >
            <X class="size-4" />
            <span class="sr-only">Clear selection</span>
          </Button>
        {/snippet}
      </Tooltip.Trigger>
      <Tooltip.Content side="top">
        <p>Clear selection</p>
      </Tooltip.Content>
    </Tooltip.Root>
  </div>
{/if}
