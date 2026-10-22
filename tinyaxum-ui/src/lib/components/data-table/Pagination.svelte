<script lang="ts" generics="TData extends RowData">
  import { Button } from '#lib/components/ui/button/index.js'
  import * as Select from '#lib/components/ui/select/index.js'
  import {
    ChevronLeft,
    ChevronRight,
    ChevronsLeft,
    ChevronsRight,
  } from '@lucide/svelte'
  import type { RowData } from '@tanstack/table-core'
  import type { AppTable } from '#lib/table-features.js'

  let { table }: { table: AppTable<TData> } = $props()

  const selectedRowsCount = $derived(
    table.getFilteredSelectedRowModel().rows.length
  )
  const totalRowsCount = $derived(
    table.options.manualPagination
      ? table.getRowCount()
      : table.getFilteredRowModel().rows.length
  )
  const pageSize = $derived(table.atoms.pagination.get().pageSize)
  const pageIndex = $derived(table.atoms.pagination.get().pageIndex)
  const pageCount = $derived(table.getPageCount())
</script>

<div class="flex items-center justify-between px-2">
  <div class="text-muted-foreground flex-1 text-sm">
    已选中 {selectedRowsCount} / {totalRowsCount} 行。
  </div>
  <div class="flex items-center space-x-6 lg:space-x-8">
    <div class="flex items-center space-x-2">
      <p class="text-sm font-medium">每页行数</p>
      <Select.Root
        type="single"
        value={String(pageSize)}
        onValueChange={(value) => table.setPageSize(Number(value))}
      >
        <Select.Trigger class="h-8 w-17.5">
          {pageSize}
        </Select.Trigger>
        <Select.Content side="top">
          {#each [10, 20, 30, 40, 50] as size}
            <Select.Item value={String(size)} label={String(size)}>
              {size}
            </Select.Item>
          {/each}
        </Select.Content>
      </Select.Root>
    </div>
    <div
      class="flex w-36 items-center justify-center text-sm font-medium whitespace-nowrap"
    >
      第 {pageIndex + 1} 页，共 {pageCount} 页
    </div>
    <div class="flex items-center space-x-2">
      <Button
        variant="outline"
        class="hidden size-8 p-0 lg:flex"
        onclick={() => table.setPageIndex(0)}
        disabled={!table.getCanPreviousPage()}
      >
        <span class="sr-only">转到第一页</span>
        <ChevronsLeft class="size-4" />
      </Button>
      <Button
        variant="outline"
        class="size-8 p-0"
        onclick={() => table.previousPage()}
        disabled={!table.getCanPreviousPage()}
      >
        <span class="sr-only">转到上一页</span>
        <ChevronLeft class="size-4" />
      </Button>
      <Button
        variant="outline"
        class="size-8 p-0"
        onclick={() => table.nextPage()}
        disabled={!table.getCanNextPage()}
      >
        <span class="sr-only">转到下一页</span>
        <ChevronRight class="size-4" />
      </Button>
      <Button
        variant="outline"
        class="hidden size-8 p-0 lg:flex"
        onclick={() => table.setPageIndex(table.getPageCount() - 1)}
        disabled={!table.getCanNextPage()}
      >
        <span class="sr-only">转到最后一页</span>
        <ChevronsRight class="size-4" />
      </Button>
    </div>
  </div>
</div>
