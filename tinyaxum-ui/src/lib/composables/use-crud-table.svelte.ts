import { untrack } from 'svelte'
import { isApiClientNotifiedError } from '$lib/api/client.js'
import { TABLE_DEFAULT_STATE } from '$lib/table-defaults.js'
import type {
  ColumnDef,
  PaginationState,
  SortingState,
  Table,
  Row,
} from '@tanstack/table-core'
import { createTable, getCoreRowModel } from '@tanstack/table-core'

export type FetchPageResult<TRow> = {
  rows: TRow[]
  totalRowCount: number
}

export type FetchPageFn<TRow> = (params: {
  pageNo: number
  pageSize: number
  keyword?: string
  sort?: string
  order?: 'asc' | 'desc'
}) => Promise<FetchPageResult<TRow>>

export type UseCrudTableOptions<TRow> = {
  columns: ColumnDef<TRow>[]
  sortableIds: Set<string>
  fetchPage: FetchPageFn<TRow>
  initialPageSize?: number
}

export type UseCrudTableReturn<TRow> = {
  table: Table<TRow>
  isLoading: boolean
  error: string | null
  globalFilter: string
  sorting: SortingState
  pagination: PaginationState
  reload: () => void
  resetPagination: () => void
}

export function useCrudTable<TRow>(
  options: UseCrudTableOptions<TRow>
): UseCrudTableReturn<TRow> {
  const { columns, sortableIds, fetchPage, initialPageSize = 10 } = options

  // State
  let tableRows = $state<TRow[]>([])
  let totalRowCount = $state(0)
  let isLoading = $state(false)
  let error = $state<string | null>(null)
  let globalFilter = $state('')
  let sorting = $state<SortingState>([])
  let pagination = $state<PaginationState>({
    pageIndex: 0,
    pageSize: initialPageSize,
  })

  // Track reload trigger
  let reloadTrigger = $state(0)

  // Fetch data effect
  $effect(() => {
    const pageNo = pagination.pageIndex + 1
    const pageSize = pagination.pageSize
    const keyword = globalFilter.trim() || undefined
    const s0 = sorting[0]
    const sort =
      s0 && sortableIds.has(String(s0.id)) ? String(s0.id) : undefined
    const order = sort ? (s0.desc ? 'desc' : 'asc') : undefined

    // Include reloadTrigger in dependencies
    void reloadTrigger

    untrack(() => {
      void doFetch({ pageNo, pageSize, keyword, sort, order })
    })
  })

  async function doFetch(params: {
    pageNo: number
    pageSize: number
    keyword?: string
    sort?: string
    order?: 'asc' | 'desc'
  }) {
    isLoading = true
    error = null
    try {
      const pageResult = await fetchPage(params)
      tableRows = pageResult.rows
      totalRowCount = pageResult.totalRowCount
    } catch (e) {
      if (isApiClientNotifiedError(e)) {
        error = null
      } else {
        error = e instanceof Error ? e.message : '加载失败'
      }
    } finally {
      isLoading = false
    }
  }

  const table = $derived(
    createTable({
      data: tableRows,
      columns,
      state: { ...TABLE_DEFAULT_STATE, sorting, globalFilter, pagination },
      onSortingChange: (updater) => {
        sorting = typeof updater === 'function' ? updater(sorting) : updater
      },
      onGlobalFilterChange: (updater) => {
        globalFilter =
          typeof updater === 'function' ? updater(globalFilter) : updater
      },
      onPaginationChange: (updater) => {
        pagination =
          typeof updater === 'function' ? updater(pagination) : updater
      },
      onStateChange: () => {},
      manualPagination: true,
      manualFiltering: true,
      manualSorting: true,
      rowCount: totalRowCount,
      getCoreRowModel: getCoreRowModel(),
      renderFallbackValue: null,
    })
  )

  function reload() {
    reloadTrigger += 1
  }

  function resetPagination() {
    pagination = { ...pagination, pageIndex: 0 }
  }

  return {
    get table() {
      return table
    },
    get isLoading() {
      return isLoading
    },
    get error() {
      return error
    },
    get globalFilter() {
      return globalFilter
    },
    set globalFilter(value: string) {
      globalFilter = value
    },
    get sorting() {
      return sorting
    },
    set sorting(value: SortingState) {
      sorting = value
    },
    get pagination() {
      return pagination
    },
    set pagination(value: PaginationState) {
      pagination = value
    },
    reload,
    resetPagination,
  }
}
