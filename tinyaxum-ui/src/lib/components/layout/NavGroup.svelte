<script lang="ts">
  import { page } from '$app/state'
  import { Badge } from '$lib/components/ui/badge/index.js'
  import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
  } from '$lib/components/ui/collapsible/index.js'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
  } from '$lib/components/ui/dropdown-menu/index.js'
  import {
    SidebarGroup,
    SidebarGroupLabel,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
    SidebarMenuSub,
    SidebarMenuSubButton,
    SidebarMenuSubItem,
    useSidebar,
  } from '$lib/components/ui/sidebar/index.js'
  import { ChevronRight } from '@lucide/svelte'
  import type {
    NavCollapsible,
    NavItem,
    NavLink,
    NavGroup as NavGroupProps,
  } from './data/types'

  let { title, items }: NavGroupProps = $props()
  let sidebar = useSidebar()

  function checkIsActive(
    href: string,
    item: NavItem & { url?: string; items?: Array<NavItem & { url: string }> },
    mainNav = false
  ): boolean {
    const itemUrl = item.url
    if (!itemUrl) return false

    return (
      href === itemUrl || // /endpoint?search=param
      href.split('?')[0] === itemUrl || // endpoint
      !!item?.items?.filter((i) => i.url === href).length || // if child nav is active
      (mainNav &&
        href.split('/')[1] !== '' &&
        href.split('/')[1] === itemUrl?.split('/')[1])
    )
  }

  function handleNavClick() {
    sidebar.setOpenMobile(false)
  }
</script>

<SidebarGroup>
  <SidebarGroupLabel>{title}</SidebarGroupLabel>
  <SidebarMenu>
    {#each items as item (item.title)}
      {#if !item.items}
        <!-- Simple link -->
        <SidebarMenuItem>
          <SidebarMenuButton
            isActive={checkIsActive(page.url.pathname, item)}
            tooltipContent={item.title}
          >
            {#snippet child({ props })}
              <a {...props} href={item.url} onclick={handleNavClick}>
                {#if item.icon}
                  <item.icon />
                {/if}
                <span>{item.title}</span>
                {#if item.badge}
                  <Badge class="rounded-full px-1 py-0 text-xs"
                    >{item.badge}</Badge
                  >
                {/if}
              </a>
            {/snippet}
          </SidebarMenuButton>
        </SidebarMenuItem>
      {:else if sidebar.state === 'collapsed' && !sidebar.isMobile}
        <!-- Collapsed dropdown -->
        <SidebarMenuItem>
          <DropdownMenu>
            <DropdownMenuTrigger>
              <SidebarMenuButton
                tooltipContent={item.title}
                isActive={checkIsActive(page.url.pathname, item)}
              >
                {#if item.icon}
                  <item.icon />
                {/if}
                <span>{item.title}</span>
                {#if item.badge}
                  <Badge class="rounded-full px-1 py-0 text-xs"
                    >{item.badge}</Badge
                  >
                {/if}
                <ChevronRight
                  class="ms-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90"
                />
              </SidebarMenuButton>
            </DropdownMenuTrigger>
            <DropdownMenuContent side="right" align="start" sideOffset={4}>
              <DropdownMenuLabel>
                {item.title}
                {item.badge ? `(${item.badge})` : ''}
              </DropdownMenuLabel>
              <DropdownMenuSeparator />
              {#each item.items as sub (sub.title)}
                <DropdownMenuItem
                  class={checkIsActive(page.url.pathname, sub)
                    ? 'bg-secondary'
                    : ''}
                  onclick={handleNavClick}
                >
                  <a href={sub.url} class="flex w-full items-center gap-2">
                    {#if sub.icon}
                      <sub.icon />
                    {/if}
                    <span class="max-w-52 text-wrap">{sub.title}</span>
                    {#if sub.badge}
                      <span class="ms-auto text-xs">{sub.badge}</span>
                    {/if}
                  </a>
                </DropdownMenuItem>
              {/each}
            </DropdownMenuContent>
          </DropdownMenu>
        </SidebarMenuItem>
      {:else}
        <!-- Collapsible group -->
        <Collapsible
          open={checkIsActive(page.url.pathname, item, true)}
          class="group/collapsible"
        >
          <SidebarMenuItem>
            <CollapsibleTrigger>
              <SidebarMenuButton tooltipContent={item.title}>
                {#if item.icon}
                  <item.icon />
                {/if}
                <span>{item.title}</span>
                {#if item.badge}
                  <Badge class="rounded-full px-1 py-0 text-xs"
                    >{item.badge}</Badge
                  >
                {/if}
                <ChevronRight
                  class="ms-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90 rtl:rotate-180"
                />
              </SidebarMenuButton>
            </CollapsibleTrigger>
            <CollapsibleContent class="CollapsibleContent">
              <SidebarMenuSub>
                {#each item.items as subItem (subItem.title)}
                  <SidebarMenuSubItem>
                    <SidebarMenuSubButton
                      href={subItem.url}
                      isActive={checkIsActive(page.url.pathname, subItem)}
                      onclick={handleNavClick}
                    >
                      {#if subItem.icon}
                        <subItem.icon />
                      {/if}
                      <span>{subItem.title}</span>
                      {#if subItem.badge}
                        <Badge class="rounded-full px-1 py-0 text-xs"
                          >{subItem.badge}</Badge
                        >
                      {/if}
                    </SidebarMenuSubButton>
                  </SidebarMenuSubItem>
                {/each}
              </SidebarMenuSub>
            </CollapsibleContent>
          </SidebarMenuItem>
        </Collapsible>
      {/if}
    {/each}
  </SidebarMenu>
</SidebarGroup>
