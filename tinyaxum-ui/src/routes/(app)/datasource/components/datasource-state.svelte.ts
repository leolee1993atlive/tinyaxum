import { isApiClientNotifiedError } from '$lib/api/client.js'
import { datasourcesApi } from '$lib/api/datasources.js'
import type { DatasourceListVo } from '$lib/api/types.js'
import {
  CrudPageState,
  createCrudContext,
} from '$lib/composables/crud-page-state.svelte.js'

export type DatasourceDialogType =
  | 'create'
  | 'update'
  | 'delete'
  | 'test'
  | 'copy'

export type DatasourceListQuery = {
  pageNo: number
  pageSize: number
  keyword?: string
  sort?: string
  order?: 'asc' | 'desc'
}

export class DatasourceState extends CrudPageState<
  DatasourceListVo,
  DatasourceDialogType
> {
  datasources = $state<DatasourceListVo[]>([])
  totalRowCount = $state(0)
  listQuery = $state<DatasourceListQuery>({ pageNo: 1, pageSize: 10 })
  isLoading = $state(false)
  error = $state<string | null>(null)
  testResult = $state<{ success: boolean; message: string } | null>(null)
  listRefetchTick = $state(0)
  copyFormData = $state<{
    code: string
    name: string
    dbType: string
    dbHost: string
    dbPort: number
    dbName: string
    dbSchema: string
    dbUsername: string
    dbPassword: string
    remark: string
  } | null>(null)

  requestListRefetch() {
    this.listRefetchTick += 1
  }

  setDatasources = (datasources: DatasourceListVo[]) => {
    this.datasources = datasources
  }

  setLoading = (loading: boolean) => {
    this.isLoading = loading
  }

  setError = (error: string | null) => {
    this.error = error
  }

  setTestResult = (result: { success: boolean; message: string } | null) => {
    this.testResult = result
  }

  setCopyFormData = (data: typeof this.copyFormData) => {
    this.copyFormData = data
  }

  /** Refetch using last query, or merge `override` then fetch (server pagination). */
  async loadDatasources(override?: Partial<DatasourceListQuery>) {
    this.isLoading = true
    this.error = null
    if (override) {
      this.listQuery = { ...this.listQuery, ...override }
    }
    const q = this.listQuery
    try {
      const pageResponse = await datasourcesApi.page({
        pageNo: q.pageNo,
        pageSize: q.pageSize,
        keyword: q.keyword,
        sort: q.sort,
        order: q.order,
      })
      this.datasources = pageResponse.data?.records ?? []
      this.totalRowCount = Number(pageResponse.data?.total ?? 0)
    } catch (e) {
      this.error = isApiClientNotifiedError(e)
        ? null
        : e instanceof Error
          ? e.message
          : 'Failed to load datasources'
    } finally {
      this.isLoading = false
    }
  }

  async testConnection(id: number) {
    try {
      const testResponse = await datasourcesApi.testConnection(id)
      const success = testResponse.data === true
      this.testResult = {
        success,
        message:
          testResponse.message ??
          (success ? 'Connection successful' : 'Connection failed'),
      }
      return success
    } catch (e) {
      this.testResult = {
        success: false,
        message: isApiClientNotifiedError(e)
          ? ''
          : e instanceof Error
            ? e.message
            : 'Connection test failed',
      }
      return false
    }
  }

  async deleteDatasource(id: number) {
    try {
      await datasourcesApi.delete(id)
      return true
    } catch (e) {
      this.error = isApiClientNotifiedError(e)
        ? null
        : e instanceof Error
          ? e.message
          : 'Failed to delete datasource'
      return false
    }
  }

  async copyDatasource(id: number) {
    try {
      const response = await datasourcesApi.getById(id)
      if (response.data) {
        const detail = response.data
        this.copyFormData = {
          code: detail.code + '_副本',
          name: detail.name,
          dbType: detail.dbType,
          dbHost: detail.dbHost,
          dbPort: detail.dbPort,
          dbName: detail.dbName,
          dbSchema: detail.dbSchema ?? 'public',
          dbUsername: detail.dbUsername,
          dbPassword: '',
          remark: detail.remark ?? '',
        }
        this.setOpen('copy')
        return true
      } else {
        this.error = '加载数据源详情失败'
        return false
      }
    } catch (e) {
      this.error = isApiClientNotifiedError(e)
        ? null
        : e instanceof Error
          ? e.message
          : '复制数据源失败'
    }
    return false
  }

  async refreshSchema(id: number) {
    try {
      await datasourcesApi.refreshSchema(id)
      await this.loadDatasources()
      return true
    } catch (e) {
      this.error = isApiClientNotifiedError(e)
        ? null
        : e instanceof Error
          ? e.message
          : '刷新元数据失败'
      return false
    }
  }
}

const { provide: provideDatasource, use: useDatasources } =
  createCrudContext<DatasourceState>('Datasource')

export { provideDatasource, useDatasources }
