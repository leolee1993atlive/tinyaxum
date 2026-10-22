<script lang="ts">
  import * as Sidebar from '#lib/components/ui/sidebar/index.js'
  import { useLayout } from '#lib/context/layout.svelte.js'
  import NavGroup from './NavGroup.svelte'
  import { sidebarData } from './data/sidebar-data'

  let layout = useLayout()

  let AppIcon = $derived(sidebarData.appIcon)
</script>

<Sidebar.Root collapsible={layout.collapsible} variant={layout.variant}>
  <Sidebar.Header>
    <div
      class="flex h-12 min-w-0 items-center gap-2 px-2 transition-[gap] duration-200 ease-linear group-data-[collapsible=icon]:gap-0"
    >
      <div
        class="bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 shrink-0 items-center justify-center rounded-lg"
      >
        <AppIcon class="size-4" />
      </div>
      <span
        class="text-sidebar-foreground max-w-[min(100%,14rem)] min-w-0 flex-1 truncate overflow-hidden text-sm font-semibold tracking-tight transition-[max-width,opacity] duration-200 ease-linear group-data-[collapsible=icon]:max-w-0 group-data-[collapsible=icon]:flex-none group-data-[collapsible=icon]:opacity-0"
        >{sidebarData.appName}</span
      >
    </div>
  </Sidebar.Header>
  <Sidebar.Content>
    {#each sidebarData.navGroups as group (group.title)}
      <NavGroup title={group.title} items={group.items} />
    {/each}
  </Sidebar.Content>
  <Sidebar.Rail />
</Sidebar.Root>
