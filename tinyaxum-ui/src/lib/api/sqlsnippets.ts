import { apiClient } from './client'
import {
  toPageQueryRecord,
  type PageResult,
  type SqlSnippetCreateRequest,
  type SqlSnippetListVo,
  type SqlSnippetPageParams,
  type SqlSnippetUpdateRequest,
} from './types'

export const sqlSnippetsApi = {
  // Get all SQL snippets
  list: () => apiClient.get<SqlSnippetListVo[]>('/sqlsnippet'),

  page: (params: SqlSnippetPageParams) => {
    const { pageNo, pageSize, sort, order, ...filters } = params
    const q = toPageQueryRecord(
      { pageNo, pageSize, sort, order },
      {
        keyword: filters.keyword,
        sqlSnippetName: filters.sqlSnippetName,
        tag: filters.tag,
      }
    )
    return apiClient.get<PageResult<SqlSnippetListVo>>('/sqlsnippet/page', q)
  },

  // Get single SQL snippet by ID
  getById: (id: number) => apiClient.get<SqlSnippetListVo>(`/sqlsnippet/${id}`),

  // Create SQL snippet
  create: (data: SqlSnippetCreateRequest) =>
    apiClient.post<void>('/sqlsnippet', data),

  // Update SQL snippet
  update: (data: SqlSnippetUpdateRequest) =>
    apiClient.put<void>('/sqlsnippet', data),

  // Soft delete SQL snippet
  delete: (id: number) => apiClient.delete<void>(`/sqlsnippet/${id}`),
}
