<script lang="ts">
  import { page } from '$app/state'
  import ConnectionAlert from '#lib/components/ConnectionAlert.svelte'
  import AppSidebar from '#lib/components/layout/AppSidebar.svelte'
  import {
    sidebarData,
    getPageTitle,
  } from '#lib/components/layout/data/sidebar-data.js'
  import {
    SidebarProvider,
    SidebarInset,
  } from '#lib/components/ui/sidebar/index.js'
  import SearchProvider from '#lib/context/SearchProvider.svelte'
  import { useAuth } from '#lib/context/auth.svelte.js'
  import { setLayoutContext } from '#lib/context/layout.svelte.js'

  let { children } = $props()

  const auth = useAuth()

  setLayoutContext()

  const pageTitle = $derived(getPageTitle(page.url.pathname))
  const docTitle = $derived(
    pageTitle ? `${pageTitle} - ${sidebarData.appName}` : sidebarData.appName
  )
</script>

<svelte:head>
  <title>{docTitle}</title>
</svelte:head>

{#if !auth.isAuthenticated}
  <div class="flex h-screen items-center justify-center">
    <div class="text-muted-foreground">Loading...</div>
  </div>
{:else}
  <SearchProvider>
    <ConnectionAlert />
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset class="min-w-0">
        {@render children()}
      </SidebarInset>
    </SidebarProvider>
  </SearchProvider>
{/if}
