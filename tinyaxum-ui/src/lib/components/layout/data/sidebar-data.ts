import { LayoutGrid, Settings, Users } from '@lucide/svelte'
import type { SidebarData } from './types'

export const sidebarData: SidebarData = {
  appName: 'TinyAxum',
  appIcon: LayoutGrid,
  navGroups: [
    {
      title: '系统管理',
      items: [
        {
          title: '用户管理',
          url: '/users',
          icon: Users,
        },
        {
          title: '系统设置',
          url: '/settings',
          icon: Settings,
        },
      ],
    },
  ],
}

/**
 * 根据当前路径从 sidebar 导航数据里反查页面标题。
 * 最长前缀匹配——`/users/123` 这种子路由也能命中 `/users`。
 * 没有匹配项返回 undefined，调用方退化到使用 appName。
 */
export function getPageTitle(pathname: string): string | undefined {
  let best: { url: string; title: string } | undefined

  const consider = (url: string, title: string) => {
    if (pathname === url || pathname.startsWith(url + '/')) {
      if (!best || url.length > best.url.length) {
        best = { url, title }
      }
    }
  }

  for (const group of sidebarData.navGroups) {
    for (const item of group.items) {
      if ('url' in item && item.url) {
        consider(item.url, item.title)
      } else if ('items' in item && item.items) {
        for (const sub of item.items) {
          consider(sub.url, sub.title)
        }
      }
    }
  }
  return best?.title
}
