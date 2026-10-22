import { goto } from '$app/navigation'
import { page } from '$app/state'
import type {
  ColumnFiltersState,
  OnChangeFn,
  PaginationState,
} from '@tanstack/table-core'

type SearchRecord = Record<string, unknown>

type UseTableUrlStateParams = {
  pagination?: {
    pageKey?: string
    pageSizeKey?: string
    defaultPage?: number
    defaultPageSize?: number
  }
  globalFilter?: {
    enabled?: boolean
    key?: string
    trim?: boolean
  }
  columnFilters?: Array<
    | {
        columnId: string
        searchKey: string
        type?: 'string'
        serialize?: (value: unknown) => string | undefined
        deserialize?: (value: string | null) => unknown
      }
    | {
        columnId: string
        searchKey: string
        type: 'array'
        serialize?: (value: unknown) => string[] | undefined
        deserialize?: (values: string[]) => unknown
      }
  >
}

export function useTableUrlState(params: UseTableUrlStateParams) {
  const {
    pagination: paginationCfg,
    globalFilter: globalFilterCfg,
    columnFilters: columnFiltersCfg = [],
  } = params

  const pageKey = paginationCfg?.pageKey ?? 'page'
  const pageSizeKey = paginationCfg?.pageSizeKey ?? 'pageSize'
  const defaultPage = paginationCfg?.defaultPage ?? 1
  const defaultPageSize = paginationCfg?.defaultPageSize ?? 10

  const globalFilterKey = globalFilterCfg?.key ?? 'filter'
  const globalFilterEnabled = globalFilterCfg?.enabled ?? true
  const trimGlobal = globalFilterCfg?.trim ?? true

  function updateUrl(
    patch: Record<string, string | string[] | number | undefined>
  ) {
    const newSearchParams = new URLSearchParams(page.url.searchParams)
    for (const [key, value] of Object.entries(patch)) {
      if (
        value === undefined ||
        value === '' ||
        (Array.isArray(value) && value.length === 0)
      ) {
        newSearchParams.delete(key)
      } else if (Array.isArray(value)) {
        newSearchParams.delete(key)
        value.forEach((v) => newSearchParams.append(key, v))
      } else {
        newSearchParams.set(key, String(value))
      }
    }
    goto(`?${newSearchParams.toString()}`, {
      replaceState: true,
      keepFocus: true,
      noScroll: true,
    })
  }

  const columnFilters = $derived.by(() => {
    const collected: ColumnFiltersState = []
    for (const cfg of columnFiltersCfg) {
      if (cfg.type === 'string') {
        const raw = page.url.searchParams.get(cfg.searchKey)
        const deserialize: (value: string | null) => unknown =
          cfg.deserialize ?? ((v: string | null) => v)
        const value = deserialize(raw)
        if (typeof value === 'string' && value.trim() !== '') {
          collected.push({ id: cfg.columnId, value })
        }
      } else {
        const raws = page.url.searchParams.getAll(cfg.searchKey)
        const deserialize: (values: string[]) => unknown =
          (cfg.type === 'array' ? cfg.deserialize : undefined) ??
          ((v: string[]) => v)
        const value = deserialize(raws)
        if (Array.isArray(value) && value.length > 0) {
          collected.push({ id: cfg.columnId, value })
        }
      }
    }
    return collected
  })

  const pagination = $derived.by(() => {
    const rawPage = page.url.searchParams.get(pageKey)
    const rawPageSize = page.url.searchParams.get(pageSizeKey)
    const pageNum = rawPage ? parseInt(rawPage) : defaultPage
    const pageSizeNum = rawPageSize ? parseInt(rawPageSize) : defaultPageSize
    return { pageIndex: Math.max(0, pageNum - 1), pageSize: pageSizeNum }
  })

  const globalFilter = $derived.by(() => {
    if (!globalFilterEnabled) return ''
    return page.url.searchParams.get(globalFilterKey) ?? ''
  })

  const onPaginationChange: OnChangeFn<PaginationState> = (updater) => {
    const next = typeof updater === 'function' ? updater(pagination) : updater
    const nextPage = next.pageIndex + 1
    const nextPageSize = next.pageSize

    updateUrl({
      [pageKey]: nextPage <= defaultPage ? undefined : nextPage,
      [pageSizeKey]:
        nextPageSize === defaultPageSize ? undefined : nextPageSize,
    })
  }

  const onGlobalFilterChange: OnChangeFn<string> = (updater) => {
    if (!globalFilterEnabled) return
    const next = typeof updater === 'function' ? updater(globalFilter) : updater
    const value = trimGlobal ? next.trim() : next

    updateUrl({
      [pageKey]: undefined,
      [globalFilterKey]: value || undefined,
    })
  }

  const onColumnFiltersChange: OnChangeFn<ColumnFiltersState> = (updater) => {
    const next =
      typeof updater === 'function' ? updater(columnFilters) : updater
    const patch: Record<string, string | string[] | number | undefined> = {
      [pageKey]: undefined,
    }

    for (const cfg of columnFiltersCfg) {
      const found = next.find((f) => f.id === cfg.columnId)
      if (cfg.type === 'string') {
        const serialize =
          cfg.serialize ?? ((v) => (typeof v === 'string' ? v : undefined))
        const value = found?.value
        patch[cfg.searchKey] = value ? serialize(value) : undefined
      } else {
        const serialize =
          cfg.serialize ??
          ((v) => (Array.isArray(v) ? (v as string[]) : undefined))
        const value = found?.value
        patch[cfg.searchKey] = value ? serialize(value) : undefined
      }
    }

    updateUrl(patch)
  }

  const ensurePageInRange = (
    pageCount: number,
    opts: { resetTo?: 'first' | 'last' } = { resetTo: 'first' }
  ) => {
    const rawPage = page.url.searchParams.get(pageKey)
    const pageNum = rawPage ? parseInt(rawPage) : defaultPage
    if (pageCount > 0 && pageNum > pageCount) {
      updateUrl({
        [pageKey]: opts.resetTo === 'last' ? pageCount : undefined,
      })
    }
  }

  return {
    get globalFilter() {
      return globalFilter
    },
    onGlobalFilterChange,
    get columnFilters() {
      return columnFilters
    },
    onColumnFiltersChange,
    get pagination() {
      return pagination
    },
    onPaginationChange,
    ensurePageInRange,
  }
}
