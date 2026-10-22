<script lang="ts">
  import {
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
  } from '$lib/components/ui/sidebar/index.js'
  import { Command } from '@lucide/svelte'
</script>

<SidebarMenu>
  <SidebarMenuItem>
    <SidebarMenuButton size="lg" class="pointer-events-none">
      <div
        class="bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg"
      >
        <Command class="size-4" />
      </div>
      <div class="grid flex-1 text-start text-sm leading-tight">
        <span class="truncate font-semibold">TinyAxum</span>
        <span class="truncate text-xs">Axum + SvelteKit</span>
      </div>
    </SidebarMenuButton>
  </SidebarMenuItem>
</SidebarMenu>
