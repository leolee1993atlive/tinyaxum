import { isApiClientNotifiedError } from '$lib/api/client.js'
import { sqlQueriesApi } from '$lib/api/sqlqueries.js'
import type { SqlQueryListVo } from '$lib/api/types.js'
import {
  CrudPageState,
  createCrudContext,
} from '$lib/composables/crud-page-state.svelte.js'

export type SQLQueryDialogType = 'delete' | 'run'

export type SQLQueryListQuery = {
  pageNo: number
  pageSize: number
  keyword?: string
  sort?: string
  order?: 'asc' | 'desc'
  dataSourceId?: number
  module?: string
  category?: string
}

export class SQLQueryState extends CrudPageState<
  SqlQueryListVo,
  SQLQueryDialogType
> {
  queries = $state<SqlQueryListVo[]>([])
  totalRowCount = $state(0)
  listQuery = $state<SQLQueryListQuery>({ pageNo: 1, pageSize: 10 })
  isLoading = $state(false)
  error = $state<string | null>(null)

  editorOpen = $state(false)
  editingQueryId = $state<number | null>(null)
  copyFromId = $state<number | null>(null)

  openEditor(queryId: number | null, copyFrom?: number | null) {
    this.editingQueryId = queryId
    this.copyFromId = copyFrom ?? null
    this.editorOpen = true
  }

  closeEditor() {
    this.editorOpen = false
    this.editingQueryId = null
    this.copyFromId = null
  }

  async loadQueries(override?: Partial<SQLQueryListQuery>) {
    this.isLoading = true
    this.error = null
    if (override) {
      this.listQuery = { ...this.listQuery, ...override }
    }
    const q = this.listQuery
    try {
      const pageResponse = await sqlQueriesApi.page({
        pageNo: q.pageNo,
        pageSize: q.pageSize,
        keyword: q.keyword,
        sort: q.sort,
        order: q.order,
        dataSourceId: q.dataSourceId,
        module: q.module,
        category: q.category,
      })
      this.queries = pageResponse.data?.records ?? []
      this.totalRowCount = Number(pageResponse.data?.total ?? 0)
    } catch (e) {
      this.error = isApiClientNotifiedError(e)
        ? null
        : e instanceof Error
          ? e.message
          : '获取查询失败'
    } finally {
      this.isLoading = false
    }
  }

  async deleteQuery(id: number) {
    try {
      await sqlQueriesApi.delete(id)
      return true
    } catch (e) {
      this.error = isApiClientNotifiedError(e)
        ? null
        : e instanceof Error
          ? e.message
          : '删除查询失败'
      return false
    }
  }
}

const { provide: provideSQLQuery, use: useSQLQuery } =
  createCrudContext<SQLQueryState>('SQLQuery')

export { provideSQLQuery, useSQLQuery }
