<script lang="ts">
  import type {
    QueryResultRow,
    SqlQueryResultFieldVo,
    SqlQueryVizTableColumn,
  } from '$lib/api/types.js'
  import { Pagination as DataTablePagination } from '$lib/components/data-table/index.js'
  import { Skeleton } from '$lib/components/ui/skeleton/index.js'
  import * as Table from '$lib/components/ui/table/index.js'
  import { TABLE_DEFAULT_STATE } from '$lib/table-defaults.js'
  import { cn } from '$lib/utils.js'
  import {
    createTable,
    getCoreRowModel,
    getPaginationRowModel,
  } from '@tanstack/table-core'
  import type { ColumnDef } from '@tanstack/table-core'

  type VizTable = {
    showRowIndex: boolean
    columns: SqlQueryVizTableColumn[]
  }

  type ColMeta = { align?: 'left' | 'center' | 'right' }

  let {
    data = [],
    loading = false,
    rowCount = 0,
    executionTime = 0,
    vizTable = undefined as VizTable | undefined,
    resultFields = [] as SqlQueryResultFieldVo[],
    columnOrder = [] as string[],
    serverTotal = undefined as number | undefined,
    onPageChange = undefined as
      | ((pageIndex: number, pageSize: number) => void)
      | undefined,
  }: {
    data: QueryResultRow[]
    loading?: boolean
    rowCount?: number
    executionTime?: number
    /** 集合查询可视化表格配置；缺省时按结果列默认渲染 */
    vizTable?: VizTable
    /** 用于表头默认标题（commentLabel） */
    resultFields?: SqlQueryResultFieldVo[]
    /** 结果列顺序（来自后端有序 columns）；缺省时回退到行对象的键序 */
    columnOrder?: string[]
    /** 服务端总条数；传入时开启服务端分页 */
    serverTotal?: number
    /** 分页变化回调（pageIndex 从 0 开始） */
    onPageChange?: (pageIndex: number, pageSize: number) => void
  } = $props()

  let pagination = $state({ pageIndex: 0, pageSize: 20 })

  function headerLabel(c: SqlQueryVizTableColumn): string {
    if (c.title?.trim()) return c.title.trim()
    const meta = resultFields.find((r) => r.fieldName === c.fieldName)
    if (meta?.commentLabel?.trim()) return meta.commentLabel.trim()
    return c.fieldName
  }

  function alignClass(m: ColMeta | undefined): string {
    const a = m?.align ?? 'left'
    if (a === 'center') return 'text-center'
    if (a === 'right') return 'text-right'
    return 'text-left'
  }

  const columns = $derived.by(() => {
    if (data.length === 0) return [] as ColumnDef<QueryResultRow, unknown>[]
    const vizCols = vizTable?.columns?.filter((c) => c.visible) ?? []
    const useViz = vizCols.length > 0
    const showRow = useViz && vizTable?.showRowIndex === true

    const defs: ColumnDef<QueryResultRow, unknown>[] = []

    if (showRow) {
      defs.push({
        id: '__rowIndex',
        header: '#',
        meta: { align: 'center' } satisfies ColMeta,
        cell: ({ row }) => {
          const global =
            pagination.pageIndex * pagination.pageSize + row.index + 1
          return String(global)
        },
      })
    }

    if (useViz) {
      for (const c of vizCols) {
        defs.push({
          accessorKey: c.fieldName,
          header: headerLabel(c),
          meta: { align: c.align } satisfies ColMeta,
          cell: (info) => {
            const val = info.getValue()
            return val === null || val === undefined ? '' : String(val)
          },
        })
      }
      return defs
    }

    // 优先用后端返回的有序列名（与 SELECT/ordinal_position 一致）；
    // 行对象是 map，其键序不可靠（后端按字母序序列化），不能用来定列序。
    const actualKeys = Object.keys(data[0])
    const keys =
      columnOrder.length > 0
        ? columnOrder.filter((k) => actualKeys.includes(k))
        : actualKeys
    return keys.map(
      (key): ColumnDef<QueryResultRow, unknown> => ({
        accessorKey: key,
        header: key,
        meta: { align: 'left' } satisfies ColMeta,
        cell: (info) => {
          const val = info.getValue()
          return val === null || val === undefined ? '' : String(val)
        },
      })
    )
  })

  let table = $derived.by(() =>
    createTable(
      serverTotal != null
        ? {
            data,
            columns,
            state: { ...TABLE_DEFAULT_STATE, pagination },
            onPaginationChange: (updater) => {
              const next =
                typeof updater === 'function' ? updater(pagination) : updater
              pagination = next
              onPageChange?.(next.pageIndex, next.pageSize)
            },
            onStateChange: () => {},
            getCoreRowModel: getCoreRowModel(),
            manualPagination: true,
            pageCount: Math.ceil(serverTotal / pagination.pageSize),
            rowCount: serverTotal,
            renderFallbackValue: null,
          }
        : {
            data,
            columns,
            state: { ...TABLE_DEFAULT_STATE, pagination },
            onPaginationChange: (updater) => {
              pagination =
                typeof updater === 'function' ? updater(pagination) : updater
            },
            onStateChange: () => {},
            getCoreRowModel: getCoreRowModel(),
            getPaginationRowModel: getPaginationRowModel(),
            renderFallbackValue: null,
          }
    )
  )

  const columnCount = $derived(columns.length > 0 ? columns.length : 5)
</script>

<div data-slot="query-results-table" class="space-y-2">
  {#if loading}
    <div class="rounded-md border">
      <Table.Root>
        <Table.Header>
          <Table.Row>
            {#each Array(columnCount) as _}
              <Table.Head>
                <Skeleton class="h-4 w-20" />
              </Table.Head>
            {/each}
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {#each Array(5) as _}
            <Table.Row>
              {#each Array(columnCount) as _}
                <Table.Cell>
                  <Skeleton class="h-4 w-full" />
                </Table.Cell>
              {/each}
            </Table.Row>
          {/each}
        </Table.Body>
      </Table.Root>
    </div>
  {:else if data.length === 0}
    <div class="rounded-md border">
      <Table.Root>
        <Table.Body>
          <Table.Row>
            <Table.Cell
              colspan={columnCount}
              class="text-muted-foreground h-24 text-center"
            >
              暂无查询结果。
            </Table.Cell>
          </Table.Row>
        </Table.Body>
      </Table.Root>
    </div>
  {:else}
    <div class="w-full overflow-x-auto rounded-md border">
      <Table.Root>
        <Table.Header>
          {#each table.getHeaderGroups() as headerGroup}
            <Table.Row>
              {#each headerGroup.headers as header}
                <Table.Head
                  class={cn(
                    'whitespace-nowrap',
                    alignClass(
                      header.column.columnDef.meta as ColMeta | undefined
                    )
                  )}
                >
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
                <Table.Cell
                  class={cn(
                    'max-w-[200px] truncate whitespace-nowrap',
                    alignClass(
                      cell.column.columnDef.meta as ColMeta | undefined
                    )
                  )}
                >
                  {cell.getValue()}
                </Table.Cell>
              {/each}
            </Table.Row>
          {/each}
        </Table.Body>
      </Table.Root>
    </div>

    <DataTablePagination {table} />
  {/if}
</div>
