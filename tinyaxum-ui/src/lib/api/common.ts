import { apiClient } from './client.js'
import type { ModuleOptionVo } from './types.js'

export const commonApi = {
  getModuleOptions: () =>
    apiClient.get<{ modules: ModuleOptionVo[] }>('/common/module-options'),
}
