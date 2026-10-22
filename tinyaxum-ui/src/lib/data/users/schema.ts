// User types matching backend UserListVo, UserCreateBo, UserUpdateBo
// Backend: hdata-analysis-rs/src/sys/user/model/user.rs

// User status type for UI display
export type UserStatus = 'active' | 'inactive' | 'invited' | 'suspended'

// Base entity fields from backend BaseVo
export interface BaseVo {
  version: number
  createdBy: number
  createdDate: string
  lastModifiedBy: number
  lastModifiedDate: string
  deletedBy?: number
  deletedDate?: string
}

// UserListVo from backend
export interface User {
  userId: number
  username: string
  nickName: string
  email?: string
  active: string
  // Flattened BaseVo fields
  version: number
  createdBy: number
  createdDate: string
  lastModifiedBy: number
  lastModifiedDate: string
  deletedBy?: number
  deletedDate?: string
}

// For creating new user (matches UserCreateBo)
export interface UserCreateInput {
  username: string
  password: string
  nickName: string
  email?: string
  active: string
}

// For updating user (matches UserUpdateBo)
export interface UserUpdateInput {
  userId: number
  username: string
  nickName: string
  email?: string
  active: string
}
