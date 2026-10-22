import { isApiClientNotifiedError } from '$lib/api/client.js'
import { usersApi } from '$lib/api/users.js'
import {
  CrudPageState,
  createCrudContext,
} from '$lib/composables/crud-page-state.svelte.js'
import type { User } from '$lib/data/users/schema.js'

export type UserDialogType = 'create' | 'update' | 'delete'

export type UserListQuery = {
  pageNo: number
  pageSize: number
  keyword?: string
  sort?: string
  order?: 'asc' | 'desc'
}

export class UsersState extends CrudPageState<User, UserDialogType> {
  users = $state<User[]>([])
  totalRowCount = $state(0)
  listQuery = $state<UserListQuery>({ pageNo: 1, pageSize: 10 })
  isLoading = $state(false)
  error = $state<string | null>(null)
  listRefetchTick = $state(0)

  requestListRefetch() {
    this.listRefetchTick += 1
  }

  async loadUsers(override?: Partial<UserListQuery>) {
    this.isLoading = true
    this.error = null
    if (override) {
      this.listQuery = { ...this.listQuery, ...override }
    }
    const q = this.listQuery
    try {
      const pageResponse = await usersApi.getUsers({
        pageNo: q.pageNo,
        pageSize: q.pageSize,
        keyword: q.keyword,
        sort: q.sort,
        order: q.order,
      })
      this.users = pageResponse.data?.records ?? []
      this.totalRowCount = Number(pageResponse.data?.total ?? 0)
    } catch (e) {
      this.error = isApiClientNotifiedError(e)
        ? null
        : e instanceof Error
          ? e.message
          : '获取用户失败'
    } finally {
      this.isLoading = false
    }
  }

  async deleteUser(id: number) {
    try {
      await usersApi.deleteUser(id)
      return true
    } catch (e) {
      this.error = isApiClientNotifiedError(e)
        ? null
        : e instanceof Error
          ? e.message
          : '删除用户失败'
      return false
    }
  }
}

const { provide: provideUsers, use: useUsers } =
  createCrudContext<UsersState>('Users')

export { provideUsers, useUsers }
