import {
  Bot,
  Database,
  FileCode,
  Globe,
  LayoutDashboard,
  LayoutGrid,
  ListTodo,
  SearchCode,
  Settings,
  Users,
} from '@lucide/svelte'
import type { SidebarData } from './types'

export const sidebarData: SidebarData = {
  appName: 'HData Analysis',
  appIcon: LayoutGrid,
  navGroups: [
    {
      title: '业务功能',
      items: [
        {
          title: '仪表板',
          url: '/dashboard',
          icon: LayoutDashboard,
        },
        {
          title: '数据源',
          url: '/datasource',
          icon: Database,
        },
        {
          title: '传输任务',
          url: '/tasks',
          icon: ListTodo,
        },
        {
          title: 'SQL 查询',
          url: '/sqlquery',
          icon: SearchCode,
        },
        {
          title: 'SQL 片段',
          url: '/sqlsnippet',
          icon: FileCode,
        },
        {
          title: 'HTTP API',
          url: '/httpapi',
          icon: Globe,
        },
        {
          title: 'AI 模型',
          url: '/ai-provider',
          icon: Bot,
        },
      ],
    },
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
 * 最长前缀匹配——`/datasource/123` 这种子路由也能命中 `/datasource`。
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
