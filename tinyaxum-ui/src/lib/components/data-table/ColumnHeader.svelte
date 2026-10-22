<script lang="ts" generics="TData extends RowData, TValue extends CellData">
  import { Button } from '#lib/components/ui/button/index.js'
  import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js'
  import { cn } from '#lib/utils.js'
  import { ArrowDown, ArrowUp, ChevronsUpDown, EyeOff } from '@lucide/svelte'
  import type { CellData, Column, RowData } from '@tanstack/table-core'
  import type { AppTableFeatures } from '#lib/table-features.js'

  let {
    column,
    title,
    class: className,
  }: {
    column: Column<AppTableFeatures, TData, TValue>
    title: string
    class?: string
  } = $props()

  const isSorted = $derived(column.getIsSorted())
</script>

{#if !column.getCanSort()}
  <div class={cn(className)}>{title}</div>
{:else}
  <div class={cn('flex items-center space-x-2', className)}>
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        {#snippet child({ props })}
          <Button
            variant="ghost"
            size="sm"
            class="data-[state=open]:bg-accent -ml-3 h-8"
            {...props}
          >
            <span>{title}</span>
            {#if isSorted === 'desc'}
              <ArrowDown class="ml-2 size-4" />
            {:else if isSorted === 'asc'}
              <ArrowUp class="ml-2 size-4" />
            {:else}
              <ChevronsUpDown class="ml-2 size-4" />
            {/if}
          </Button>
        {/snippet}
      </DropdownMenu.Trigger>
      <DropdownMenu.Content align="start">
        <DropdownMenu.Item onSelect={() => column.toggleSorting(false)}>
          <ArrowUp class="text-muted-foreground/70 mr-2 size-3.5" />
          Asc
        </DropdownMenu.Item>
        <DropdownMenu.Item onSelect={() => column.toggleSorting(true)}>
          <ArrowDown class="text-muted-foreground/70 mr-2 size-3.5" />
          Desc
        </DropdownMenu.Item>
        <DropdownMenu.Separator />
        <DropdownMenu.Item onSelect={() => column.toggleVisibility(false)}>
          <EyeOff class="text-muted-foreground/70 mr-2 size-3.5" />
          Hide
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  </div>
{/if}
