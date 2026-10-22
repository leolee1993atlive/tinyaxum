import { aiProvidersApi } from '$lib/api/ai-providers.js'
import { isApiClientNotifiedError } from '$lib/api/client.js'
import type { AiProviderListVo } from '$lib/api/types.js'
import {
  CrudPageState,
  createCrudContext,
} from '$lib/composables/crud-page-state.svelte.js'

export type AiProviderDialogType = 'create' | 'update' | 'delete' | 'copy'

export class AiProviderState extends CrudPageState<
  AiProviderListVo,
  AiProviderDialogType
> {
  providers = $state<AiProviderListVo[]>([])
  isLoading = $state(false)
  error = $state<string | null>(null)
  listRefetchTick = $state(0)
  copyProviderData = $state<AiProviderListVo | null>(null)

  requestListRefetch() {
    this.listRefetchTick += 1
  }

  async loadProviders(): Promise<{
    rows: AiProviderListVo[]
    totalRowCount: number
  }> {
    this.isLoading = true
    this.error = null
    try {
      const res = await aiProvidersApi.list()
      this.providers = res.data ?? []
      return { rows: this.providers, totalRowCount: this.providers.length }
    } catch (e) {
      this.error = isApiClientNotifiedError(e)
        ? null
        : e instanceof Error
          ? e.message
          : '加载失败'
      return { rows: [], totalRowCount: 0 }
    } finally {
      this.isLoading = false
    }
  }

  async deleteProvider(id: number) {
    try {
      await aiProvidersApi.delete(id)
      return true
    } catch (e) {
      this.error = isApiClientNotifiedError(e)
        ? null
        : e instanceof Error
          ? e.message
          : '删除失败'
      return false
    }
  }

  async setDefault(id: number) {
    try {
      await aiProvidersApi.setDefault(id)
      return true
    } catch (e) {
      this.error = isApiClientNotifiedError(e)
        ? null
        : e instanceof Error
          ? e.message
          : '操作失败'
      return false
    }
  }

  async copyProvider(id: number) {
    try {
      const res = await aiProvidersApi.getById(id)
      if (res.data) {
        this.copyProviderData = {
          ...res.data,
          name: res.data.name + '_副本',
        }
        this.setOpen('copy')
        return true
      } else {
        this.error = '加载模型详情失败'
        return false
      }
    } catch (e) {
      this.error = isApiClientNotifiedError(e)
        ? null
        : e instanceof Error
          ? e.message
          : '复制模型失败'
      return false
    }
  }

  resetCopyProvider() {
    this.copyProviderData = null
  }

  async testProvider(id: number) {
    try {
      const res = await aiProvidersApi.test(id)
      return res.data ?? null
    } catch {
      return null
    }
  }
}

const { provide: provideAiProvider, use: useAiProviders } =
  createCrudContext<AiProviderState>('AiProvider')

export { provideAiProvider, useAiProviders }
