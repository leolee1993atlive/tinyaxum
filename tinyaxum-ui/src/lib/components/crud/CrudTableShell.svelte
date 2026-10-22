<script lang="ts" generics="TRow extends RowData">
  import type { Snippet } from 'svelte'
  import { Pagination as DataTablePagination } from '#lib/components/data-table/index.js'
  import { Button } from '#lib/components/ui/button/index.js'
  import { Input } from '#lib/components/ui/input/index.js'
  import * as Skeleton from '#lib/components/ui/skeleton/index.js'
  import * as Table from '#lib/components/ui/table/index.js'
  import { RefreshCw, Search } from '@lucide/svelte'
  import type { RowData } from '@tanstack/table-core'
  import type { AppTable } from '#lib/table-features.js'

  type Props = {
    table: AppTable<TRow>
    isLoading: boolean
    error: string | null
    columnCount: number
    searchPlaceholder?: string
    emptyMessage?: string
    globalFilter: string
    onSearchInput?: (value: string) => void
    onRefresh?: () => void
    cellRenderer: Snippet<[{ row: TRow; columnId?: string }]>
    customCellIds?: Set<string>
    extraToolbar?: Snippet
  }

  let {
    table,
    isLoading,
    error,
    columnCount,
    searchPlaceholder = '搜索...',
    emptyMessage = '未找到数据。',
    globalFilter = $bindable(),
    onSearchInput,
    onRefresh,
    cellRenderer,
    customCellIds,
    extraToolbar,
  }: Props = $props()

  const columns = $derived(table.getAllColumns())

  function handleInput(event: Event) {
    const value = (event.target as HTMLInputElement).value
    globalFilter = value
    onSearchInput?.(value)
  }
</script>

<div class="space-y-4">
  <div class="flex flex-wrap items-center gap-2">
    <div class="relative max-w-md min-w-0 flex-1">
      <Search class="text-muted-foreground absolute top-2.5 left-2.5 h-4 w-4" />
      <Input
        placeholder={searchPlaceholder}
        class="pl-9"
        type="search"
        value={globalFilter}
        oninput={handleInput}
      />
    </div>
    {#if extraToolbar}
      {@render extraToolbar()}
    {/if}
    {#if onRefresh}
      <Button
        type="button"
        variant="outline"
        size="icon"
        title="刷新列表"
        aria-label="刷新列表"
        onclick={() => onRefresh()}
      >
        <RefreshCw class="h-4 w-4" />
      </Button>
    {/if}
  </div>

  {#if isLoading}
    <div class="rounded-md border">
      <Table.Root>
        <Table.Header>
          <Table.Row>
            {#each Array(columnCount) as _}
              <Table.Head><Skeleton.Root class="h-4 w-16" /></Table.Head>
            {/each}
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {#each Array(5) as _}
            <Table.Row>
              {#each Array(columnCount) as _}
                <Table.Cell><Skeleton.Root class="h-4 w-full" /></Table.Cell>
              {/each}
            </Table.Row>
          {/each}
        </Table.Body>
      </Table.Root>
    </div>
  {:else if error}
    <div class="flex h-64 items-center justify-center">
      <div class="text-destructive">{error}</div>
    </div>
  {:else}
    <div class="rounded-md border">
      <Table.Root>
        <Table.Header>
          {#each table.getHeaderGroups() as headerGroup}
            <Table.Row>
              {#each headerGroup.headers as header}
                <Table.Head>
                  {#if !header.isPlaceholder}
                    {header.column.columnDef.header}
                  {/if}
                </Table.Head>
              {/each}
            </Table.Row>
          {/each}
        </Table.Header>
        <Table.Body>
          {#each table.getRowModel().rows as row}
            <Table.Row>
              {#each row.getVisibleCells() as cell}
                <Table.Cell class={cell.column.columnDef.meta?.cellClass}>
                  {#if cell.column.id === 'actions' || (customCellIds && customCellIds.has(cell.column.id))}
                    {@render cellRenderer({
                      row: row.original,
                      columnId: cell.column.id,
                    })}
                  {:else}
                    {cell.getValue()}
                  {/if}
                </Table.Cell>
              {/each}
            </Table.Row>
          {:else}
            <Table.Row>
              <Table.Cell colspan={columnCount} class="h-24 text-center">
                {emptyMessage}
              </Table.Cell>
            </Table.Row>
          {/each}
        </Table.Body>
      </Table.Root>
    </div>

    <DataTablePagination {table} />
  {/if}
</div>
