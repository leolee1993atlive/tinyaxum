import type { Component } from 'svelte'

// Navigation types
export type NavItem = {
  title: string
  badge?: string
  icon?: Component
}

export type NavLink = NavItem & {
  url: string
  items?: never
}

export type NavCollapsible = NavItem & {
  items: Array<NavItem & { url: string }>
  url?: never
}

export type NavGroup = {
  title: string
  items: Array<NavLink | NavCollapsible>
}

export type SidebarData = {
  appName: string
  appIcon: Component
  navGroups: NavGroup[]
}
