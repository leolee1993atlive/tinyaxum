import { httpapiApi } from '$lib/api/httpapi.js'
import {
  CrudPageState,
  createCrudContext,
} from '$lib/composables/crud-page-state.svelte.js'
import type {
  HttpApi,
  HttpApiCreateInput,
  HttpApiDetail,
  HttpApiUpdateInput,
} from '$lib/data/httpapi/schema.js'
import { toast } from 'svelte-sonner'

export type HttpApiDialogType = 'create' | 'edit' | 'copy'

export class HttpApiState extends CrudPageState<HttpApi, HttpApiDialogType> {
  list = $state<HttpApi[]>([])
  loading = $state(false)
  currentApi = $state<HttpApiDetail | null>(null)
  listRefetchTick = $state(0)
  copyApiData = $state<HttpApiDetail | null>(null)

  requestListRefetch() {
    this.listRefetchTick += 1
  }

  async loadList() {
    this.loading = true
    try {
      const res = await httpapiApi.getList()
      if (res.data) {
        this.list = res.data
      }
    } catch {
      toast.error('加载列表失败')
    } finally {
      this.loading = false
    }
  }

  async loadPage(params: {
    pageNo?: number
    pageSize?: number
    keyword?: string
    enabled?: number
    module?: string
    category?: string
  }) {
    this.loading = true
    try {
      const res = await httpapiApi.getPage(params)
      if (res.data) {
        this.list = res.data.records
        return res.data
      }
    } catch {
      toast.error('加载列表失败')
    } finally {
      this.loading = false
    }
    return null
  }

  async loadDetail(id: number) {
    try {
      const res = await httpapiApi.getById(id)
      if (res.data) {
        this.currentApi = res.data
        return res.data
      }
    } catch {
      toast.error('加载详情失败')
    }
    return null
  }

  async createApi(data: HttpApiCreateInput) {
    try {
      const res = await httpapiApi.create(data)
      if (res.code === 200) {
        toast.success('创建成功')
        return true
      } else {
        toast.error(res.message || '创建失败')
      }
    } catch {
      toast.error('创建失败')
    }
    return false
  }

  async updateApi(data: HttpApiUpdateInput) {
    try {
      const res = await httpapiApi.update(data)
      if (res.code === 200) {
        toast.success('更新成功')
        return true
      } else {
        toast.error(res.message || '更新失败')
      }
    } catch {
      toast.error('更新失败')
    }
    return false
  }

  async deleteApi(id: number) {
    try {
      const res = await httpapiApi.delete(id)
      if (res.code === 200) {
        toast.success('删除成功')
        return true
      } else {
        toast.error(res.message || '删除失败')
      }
    } catch {
      toast.error('删除失败')
    }
    return false
  }

  async toggleEnabled(id: number, enabled: boolean) {
    try {
      const res = enabled
        ? await httpapiApi.enable(id)
        : await httpapiApi.disable(id)
      if (res.code === 200) {
        toast.success(enabled ? '已启用' : '已禁用')
        return true
      } else {
        toast.error(res.message || '操作失败')
      }
    } catch {
      toast.error('操作失败')
    }
    return false
  }

  async copyApi(id: number) {
    const detail = await this.loadDetail(id)
    if (detail) {
      this.copyApiData = {
        ...detail,
        httpApiName: detail.httpApiName + '_副本',
      }
      this.setOpen('copy')
      return true
    } else {
      toast.error('加载API详情失败')
      return false
    }
  }

  resetCopyApi() {
    this.copyApiData = null
  }
}

const { provide: provideHttpApi, use: useHttpApi } =
  createCrudContext<HttpApiState>('HttpApi')

export { provideHttpApi, useHttpApi }
