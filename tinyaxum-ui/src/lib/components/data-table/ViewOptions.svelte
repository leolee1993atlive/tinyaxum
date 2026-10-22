<script lang="ts" generics="TData">
  import { Button } from '$lib/components/ui/button/index.js'
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js'
  import { Settings2 } from '@lucide/svelte'
  import type { Table } from '@tanstack/table-core'

  let { table }: { table: Table<TData> } = $props()

  const columns = $derived(
    table
      .getAllColumns()
      .filter(
        (column) =>
          typeof column.accessorFn !== 'undefined' && column.getCanHide()
      )
  )
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger>
    {#snippet child({ props })}
      <Button
        variant="outline"
        size="sm"
        class="ml-auto hidden h-8 lg:flex"
        {...props}
      >
        <Settings2 class="mr-2 size-4" />
        View
      </Button>
    {/snippet}
  </DropdownMenu.Trigger>
  <DropdownMenu.Content align="end" class="w-[150px]">
    <DropdownMenu.Label>Toggle columns</DropdownMenu.Label>
    <DropdownMenu.Separator />
    {#each columns as column}
      <DropdownMenu.CheckboxItem
        class="capitalize"
        checked={column.getIsVisible()}
        onCheckedChange={(value) => column.toggleVisibility(!!value)}
      >
        {column.id}
      </DropdownMenu.CheckboxItem>
    {/each}
  </DropdownMenu.Content>
</DropdownMenu.Root>
