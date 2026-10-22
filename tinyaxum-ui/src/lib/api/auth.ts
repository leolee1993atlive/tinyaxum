import { apiClient } from './client'
import type { LoginRequest, LoginResponse } from './types'

export const authApi = {
  login: (data: LoginRequest) => apiClient.post<LoginResponse>('/login', data),
}
