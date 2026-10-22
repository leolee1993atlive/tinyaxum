import type {
  HttpApi,
  HttpApiCreateInput,
  HttpApiDetail,
  HttpApiUpdateInput,
} from '$lib/data/httpapi/schema.js'
import { apiClient } from './client'
import type { ApiResponse, PageResult } from './types'
import { toPageQueryRecord } from './types'

export interface HttpApiPageParams {
  pageNo?: number
  pageSize?: number
  keyword?: string
  enabled?: number
  module?: string
  category?: string
}

export const httpapiApi = {
  getList: () => apiClient.get<HttpApi[]>('/httpapi'),

  getPage: (params: HttpApiPageParams) => {
    const q = toPageQueryRecord(
      { pageNo: params.pageNo ?? 1, pageSize: params.pageSize ?? 10 },
      {
        keyword: params.keyword,
        enabled: params.enabled,
        module: params.module,
        category: params.category,
      }
    )
    return apiClient.get<PageResult<HttpApi>>('/httpapi/page', q)
  },

  getById: (id: number) => apiClient.get<HttpApiDetail>(`/httpapi/${id}`),

  create: (data: HttpApiCreateInput) =>
    apiClient.post<ApiResponse<null>>('/httpapi', data),

  update: (data: HttpApiUpdateInput) =>
    apiClient.put<ApiResponse<null>>('/httpapi', data),

  delete: (id: number) => apiClient.delete<ApiResponse<null>>(`/httpapi/${id}`),

  enable: (id: number) =>
    apiClient.put<ApiResponse<null>>(`/httpapi/${id}/enable`, {}),

  disable: (id: number) =>
    apiClient.put<ApiResponse<null>>(`/httpapi/${id}/disable`, {}),
}
