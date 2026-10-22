import type { DashboardConfigItem, SqlQueryListVo } from '$lib/api/types.js'

export type GroupedCategory = {
  category: string
  items: DashboardConfigItem[]
}

export type GroupedModule = {
  module: string
  directItems: DashboardConfigItem[]
  categories: GroupedCategory[]
}

export type GroupedItems = {
  noModule: DashboardConfigItem[]
  modules: GroupedModule[]
}

export type QueryGroupMeta = Pick<SqlQueryListVo, 'module' | 'category'>

export function groupDashboardItems(
  items: DashboardConfigItem[],
  metaById: Map<number, QueryGroupMeta>
): GroupedItems {
  const noModule: DashboardConfigItem[] = []
  const modules: GroupedModule[] = []
  const moduleIndex = new Map<string, GroupedModule>()

  for (const item of items) {
    const meta = metaById.get(item.sqlQueryId)
    const moduleName = (meta?.module ?? '').trim()
    const categoryName = (meta?.category ?? '').trim()

    if (!moduleName) {
      noModule.push(item)
      continue
    }

    let mod = moduleIndex.get(moduleName)
    if (!mod) {
      mod = { module: moduleName, directItems: [], categories: [] }
      moduleIndex.set(moduleName, mod)
      modules.push(mod)
    }

    if (!categoryName) {
      mod.directItems.push(item)
      continue
    }

    let cat = mod.categories.find((c) => c.category === categoryName)
    if (!cat) {
      cat = { category: categoryName, items: [] }
      mod.categories.push(cat)
    }
    cat.items.push(item)
  }

  return { noModule, modules }
}

export function moduleItemCount(mod: GroupedModule): number {
  return (
    mod.directItems.length +
    mod.categories.reduce((sum, c) => sum + c.items.length, 0)
  )
}
