import { apiClient } from './client'
import type {
  AiProviderCreateRequest,
  AiProviderListVo,
  AiProviderTestResultVo,
  AiProviderUpdateRequest,
} from './types'

export const aiProvidersApi = {
  list: () => apiClient.get<AiProviderListVo[]>('/ai/providers'),
  getById: (id: number) =>
    apiClient.get<AiProviderListVo>(`/ai/providers/${id}`),
  create: (data: AiProviderCreateRequest) =>
    apiClient.post<number>('/ai/providers', data),
  update: (id: number, data: AiProviderUpdateRequest) =>
    apiClient.put<void>(`/ai/providers/${id}`, data),
  delete: (id: number) => apiClient.delete<void>(`/ai/providers/${id}`),
  test: (id: number) =>
    apiClient.post<AiProviderTestResultVo>(`/ai/providers/${id}/test`, {}),
  setDefault: (id: number) =>
    apiClient.put<void>(`/ai/providers/${id}/default`, {}),
}
