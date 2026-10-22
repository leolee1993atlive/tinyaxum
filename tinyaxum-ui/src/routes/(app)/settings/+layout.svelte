<script lang="ts">
  import ProfileDropdown from '#lib/components/ProfileDropdown.svelte'
  import Search from '#lib/components/Search.svelte'
  import Header from '#lib/components/layout/Header.svelte'
  import Main from '#lib/components/layout/Main.svelte'
  import SidebarNav from '#lib/components/settings/SidebarNav.svelte'
  import { Separator } from '#lib/components/ui/separator/index.js'
  import { UserCog, Wrench, Palette, Bell, Monitor } from '@lucide/svelte'

  const sidebarNavItems = [
    { title: '个人资料', href: '/settings', icon: UserCog },
    { title: '账户', href: '/settings/account', icon: Wrench },
    { title: '外观', href: '/settings/appearance', icon: Palette },
    { title: '通知', href: '/settings/notifications', icon: Bell },
    { title: '显示', href: '/settings/display', icon: Monitor },
  ]
</script>

<Header>
  <Search />
  <ProfileDropdown />
</Header>

<Main fixed>
  <div class="space-y-0.5">
    <h1 class="text-2xl font-bold tracking-tight md:text-3xl">设置</h1>
    <p class="text-muted-foreground">管理您的账户设置和邮件偏好。</p>
  </div>
  <Separator class="my-4 lg:my-6" />
  <div
    class="flex flex-1 flex-col space-y-2 overflow-hidden md:space-y-2 lg:flex-row lg:space-y-0 lg:space-x-12"
  >
    <aside class="top-0 lg:sticky lg:w-1/5">
      <SidebarNav items={sidebarNavItems} />
    </aside>
    <div class="flex w-full overflow-y-hidden p-1">
      <slot />
    </div>
  </div>
</Main>
