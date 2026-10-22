import { isApiClientNotifiedError } from '$lib/api/client.js'
import { sqlSnippetsApi } from '$lib/api/sqlsnippets.js'
import type { SqlSnippetListVo } from '$lib/api/types.js'
import {
  CrudPageState,
  createCrudContext,
} from '$lib/composables/crud-page-state.svelte.js'

export type SqlSnippetDialogType = 'create' | 'update' | 'delete' | 'copy'

export type SqlSnippetListQuery = {
  pageNo: number
  pageSize: number
  keyword?: string
  sort?: string
  order?: 'asc' | 'desc'
}

export class SqlSnippetState extends CrudPageState<
  SqlSnippetListVo,
  SqlSnippetDialogType
> {
  snippets = $state<SqlSnippetListVo[]>([])
  totalRowCount = $state(0)
  listQuery = $state<SqlSnippetListQuery>({ pageNo: 1, pageSize: 10 })
  isLoading = $state(false)
  error = $state<string | null>(null)
  copySnippetData = $state<SqlSnippetListVo | null>(null)

  async loadSnippets(override?: Partial<SqlSnippetListQuery>) {
    this.isLoading = true
    this.error = null
    if (override) {
      this.listQuery = { ...this.listQuery, ...override }
    }
    const q = this.listQuery
    try {
      const pageResponse = await sqlSnippetsApi.page({
        pageNo: q.pageNo,
        pageSize: q.pageSize,
        keyword: q.keyword,
        sort: q.sort,
        order: q.order,
      })
      this.snippets = pageResponse.data?.records ?? []
      this.totalRowCount = Number(pageResponse.data?.total ?? 0)
    } catch (e) {
      this.error = isApiClientNotifiedError(e)
        ? null
        : e instanceof Error
          ? e.message
          : '获取片段失败'
    } finally {
      this.isLoading = false
    }
  }

  async deleteSnippet(id: number) {
    try {
      await sqlSnippetsApi.delete(id)
      return true
    } catch (e) {
      this.error = isApiClientNotifiedError(e)
        ? null
        : e instanceof Error
          ? e.message
          : '删除片段失败'
      return false
    }
  }

  async copySnippet(id: number) {
    try {
      const response = await sqlSnippetsApi.getById(id)
      if (response.data) {
        this.copySnippetData = {
          ...response.data,
          sqlSnippetName: response.data.sqlSnippetName + '_副本',
        }
        this.setOpen('copy')
        return true
      } else {
        this.error = '加载片段详情失败'
        return false
      }
    } catch (e) {
      this.error = isApiClientNotifiedError(e)
        ? null
        : e instanceof Error
          ? e.message
          : '复制片段失败'
      return false
    }
  }

  resetCopySnippet() {
    this.copySnippetData = null
  }
}

const { provide: provideSqlSnippets, use: useSqlSnippets } =
  createCrudContext<SqlSnippetState>('SqlSnippet')

export { provideSqlSnippets, useSqlSnippets }
