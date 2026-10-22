<script lang="ts" generics="TData">
  import { Button } from '$lib/components/ui/button/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import { X } from '@lucide/svelte'
  import type { Table } from '@tanstack/table-core'
  import FacetedFilter from './FacetedFilter.svelte'
  import ViewOptions from './ViewOptions.svelte'

  let {
    table,
    searchPlaceholder = '筛选...',
    searchKey,
    filters = [],
  }: {
    table: Table<TData>
    searchPlaceholder?: string
    searchKey?: string
    filters?: {
      columnId: string
      title: string
      options: {
        label: string
        value: string
        icon?: import('svelte').ComponentType
      }[]
    }[]
  } = $props()

  const isFiltered = $derived(
    table.getState().columnFilters.length > 0 || table.getState().globalFilter
  )
</script>

<div class="flex items-center justify-between">
  <div
    class="flex flex-1 flex-col-reverse items-start gap-y-2 sm:flex-row sm:items-center sm:space-x-2"
  >
    {#if searchKey}
      <Input
        placeholder={searchPlaceholder}
        value={String(table.getColumn(searchKey)?.getFilterValue() ?? '')}
        oninput={(e) =>
          table.getColumn(searchKey)?.setFilterValue(e.currentTarget.value)}
        class="h-8 w-[150px] lg:w-[250px]"
      />
    {:else}
      <Input
        placeholder={searchPlaceholder}
        value={table.getState().globalFilter ?? ''}
        oninput={(e) => table.setGlobalFilter(e.currentTarget.value)}
        class="h-8 w-[150px] lg:w-[250px]"
      />
    {/if}

    <div class="flex gap-x-2">
      {#each filters as filter}
        {@const column = table.getColumn(filter.columnId)}
        {#if column}
          <FacetedFilter
            {column}
            title={filter.title}
            options={filter.options}
          />
        {/if}
      {/each}
    </div>

    {#if isFiltered}
      <Button
        variant="ghost"
        onclick={() => {
          table.resetColumnFilters()
          table.setGlobalFilter('')
        }}
        class="h-8 px-2 lg:px-3"
      >
        重置
        <X class="ml-2 size-4" />
      </Button>
    {/if}
  </div>
  <ViewOptions {table} />
</div>
