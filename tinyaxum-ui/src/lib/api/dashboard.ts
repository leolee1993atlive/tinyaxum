import { apiClient } from './client'
import type {
  DashboardConfigSaveRequest,
  DashboardConfigVo,
  DashboardRunVo,
} from './types'

export const dashboardApi = {
  getConfig: () => apiClient.get<DashboardConfigVo>('/dashboard/config'),
  saveConfig: (data: DashboardConfigSaveRequest) =>
    apiClient.put<void>('/dashboard/config', data),

  runCached: (
    id: number,
    month: string,
    params?: Record<string, string>,
    opts?: { noCache?: boolean }
  ) => {
    const q: Record<string, string> = { month, ...(params ?? {}) }
    if (opts?.noCache) q.noCache = 'true'
    return apiClient.get<DashboardRunVo>(`/dashboard/run/${id}`, q)
  },

  invalidateMonth: (month: string) =>
    apiClient.post<void>(
      `/dashboard/cache/invalidate?month=${encodeURIComponent(month)}`,
      {}
    ),
}
