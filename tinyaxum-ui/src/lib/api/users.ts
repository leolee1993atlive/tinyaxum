import type { User } from '#lib/data/users/schema.js'
import { apiClient } from './client'
import {
  toPageQueryRecord,
  type PageResult,
  type UserCreateRequest,
  type UserPageParams,
  type UserUpdateRequest,
} from './types'

export interface UserInfo {
  userId: number
  username: string
}

export const usersApi = {
  /** Paginated list (backend: GET /user/page) */
  getUsers: (params: UserPageParams) => {
    const { pageNo, pageSize, sort, order, ...filters } = params
    const q = toPageQueryRecord(
      { pageNo, pageSize, sort, order },
      {
        keyword: filters.keyword,
        username: filters.username,
        nickName: filters.nickName,
        active: filters.active,
      }
    )
    return apiClient.get<PageResult<User>>('/user/page', q)
  },

  // Get current user info (backend: GET /user/info)
  getCurrentUser: () => apiClient.get<UserInfo>('/user/info'),

  // Get single user (backend: GET /user/{id})
  getUser: (id: number) => apiClient.get<User>(`/user/${id}`),

  // Create user (backend: POST /user)
  createUser: (data: UserCreateRequest) => apiClient.post<User>('/user', data),

  // Update user (backend: PUT /user)
  updateUser: (data: UserUpdateRequest) => apiClient.put<User>('/user', data),

  // Soft delete user (backend: DELETE /user/{id})
  deleteUser: (id: number) => apiClient.delete<void>(`/user/${id}`),

  // Hard delete user (backend: DELETE /user/delete/{id})
  trueDeleteUser: (id: number) => apiClient.delete<void>(`/user/delete/${id}`),

  // Update password (backend: PUT /user/password)
  updatePassword: (userId: number, password: string) =>
    apiClient.put<void>('/user/password', { userId, password }),
}
