import { apiClient } from './client'
import {
  toPageQueryRecord,
  type DatasourceSchemaVo,
  type PageResult,
  type SqlQueryCreateRequest,
  type SqlQueryDetailVo,
  type SqlQueryListVo,
  type SqlQueryPageParams,
  type SqlQueryRunAdhocRequest,
  type SqlQueryUpdateRequest,
  type SqlQueryVizConfigSaveRequest,
  type SqlRunResultVo,
} from './types'

export const sqlQueriesApi = {
  // Get all SQL queries
  list: () => apiClient.get<SqlQueryListVo[]>('/sqlquery'),

  page: (params: SqlQueryPageParams) => {
    const { pageNo, pageSize, sort, order, ...filters } = params
    const q = toPageQueryRecord(
      { pageNo, pageSize, sort, order },
      {
        keyword: filters.keyword,
        sqlQueryName: filters.sqlQueryName,
        dataSourceId: filters.dataSourceId,
        module: filters.module,
        category: filters.category,
      }
    )
    return apiClient.get<PageResult<SqlQueryListVo>>('/sqlquery/page', q)
  },

  // Get single SQL query by ID
  getById: (id: number) => apiClient.get<SqlQueryDetailVo>(`/sqlquery/${id}`),

  // Create SQL query
  create: (data: SqlQueryCreateRequest) =>
    apiClient.post<number>('/sqlquery', data),

  // Update SQL query
  update: (data: SqlQueryUpdateRequest) =>
    apiClient.put<void>('/sqlquery', data),

  /** 仅更新可视化配置 JSON（`PUT /sqlquery/{id}/viz-config`） */
  saveVizConfig: (id: number, data: SqlQueryVizConfigSaveRequest) =>
    apiClient.put<void>(`/sqlquery/${id}/viz-config`, data),

  // Soft delete SQL query
  delete: (id: number) => apiClient.delete<void>(`/sqlquery/${id}`),

  // Run SQL query by ID
  run: (
    id: number,
    params?: Record<string, string>,
    page?: { pageNo?: number; pageSize?: number }
  ) => {
    const q: Record<string, string> = { ...params }
    if (page?.pageNo != null) q.pageNo = String(page.pageNo)
    if (page?.pageSize != null) q.pageSize = String(page.pageSize)
    return apiClient.get<SqlRunResultVo>(`/sqlquery/run/${id}`, q)
  },

  // Run ad-hoc SQL query
  runAdhoc: (data: SqlQueryRunAdhocRequest) =>
    apiClient.post<SqlRunResultVo>('/sqlquery/run-adhoc', data),

  // Validate SQL syntax against source database
  validate: (sqlQueryContent: string, dataSourceId: number) =>
    apiClient.post<string | null>('/sqlquery/validate', {
      sqlQueryContent,
      dataSourceId,
    }),

  /** SQL 编辑器专用:只返回已导出到 DuckDB 的表元数据(GET /sqlquery/schema/{id}) */
  getQuerySchema: (id: number) =>
    apiClient.get<DatasourceSchemaVo>(`/sqlquery/schema/${id}`),
}
