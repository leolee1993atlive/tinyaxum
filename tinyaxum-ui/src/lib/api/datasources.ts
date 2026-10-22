import { apiClient } from './client'
import type { ApiResponse } from './types'
import {
  toPageQueryRecord,
  type DatasourceCreateRequest,
  type DatasourceDetailVo,
  type DatasourceListVo,
  type DatasourcePageParams,
  type DatasourceSchemaVo,
  type DatasourceUpdateRequest,
  type PageResult,
} from './types'

export const datasourcesApi = {
  // Get all datasources
  list: () => apiClient.get<DatasourceListVo[]>('/datasource'),

  page: (params: DatasourcePageParams) => {
    const { pageNo, pageSize, sort, order, ...filters } = params
    const q = toPageQueryRecord(
      { pageNo, pageSize, sort, order },
      {
        keyword: filters.keyword,
        code: filters.code,
        name: filters.name,
        dbType: filters.dbType,
      }
    )
    return apiClient.get<PageResult<DatasourceListVo>>('/datasource/page', q)
  },

  // Get single datasource by ID (returns detail with password)
  getById: (id: number) =>
    apiClient.get<DatasourceDetailVo>(`/datasource/${id}`),

  // Create datasource
  create: async (data: DatasourceCreateRequest): Promise<boolean> => {
    const response = await apiClient.post<DatasourceListVo>('/datasource', data)
    return response.code === 200
  },

  // Update datasource
  update: async (data: DatasourceUpdateRequest): Promise<boolean> => {
    const response = await apiClient.put<DatasourceListVo>('/datasource', data)
    return response.code === 200
  },

  // Soft delete datasource
  delete: (id: number) => apiClient.delete<void>(`/datasource/${id}`),

  // Test datasource connection
  testConnection: (id: number) =>
    apiClient.get<boolean>(`/datasource/test/${id}`),

  /** 已缓存的表/列元数据 */
  getSchema: (id: number) =>
    apiClient.get<DatasourceSchemaVo>(`/datasource/${id}/schema`),

  /** 从源库重新采集并覆盖元数据 */
  refreshSchema: (id: number) =>
    apiClient.post<void>(`/datasource/${id}/schema/refresh`, {}),
}
