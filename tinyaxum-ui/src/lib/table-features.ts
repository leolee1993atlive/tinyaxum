import {
  columnFacetingFeature,
  columnFilteringFeature,
  columnVisibilityFeature,
  globalFilteringFeature,
  rowPaginationFeature,
  rowSelectionFeature,
  rowSortingFeature,
  tableFeatures,
} from '@tanstack/table-core'
import type { RowData, Table } from '@tanstack/table-core'

/**
 * The table features actually used by the CRUD table stack.
 *
 * Keep this object static: features are registered once per table instance and
 * their keys define which state slices and APIs exist on `table`/`column`.
 */
export const appTableFeatures = tableFeatures({
  columnFacetingFeature,
  columnFilteringFeature,
  columnVisibilityFeature,
  globalFilteringFeature,
  rowPaginationFeature,
  rowSelectionFeature,
  rowSortingFeature,
})

export type AppTableFeatures = typeof appTableFeatures

export type AppTable<TData extends RowData> = Table<AppTableFeatures, TData>
