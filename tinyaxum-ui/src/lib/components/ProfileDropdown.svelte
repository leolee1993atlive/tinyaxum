<script lang="ts">
  import { goto } from '$app/navigation'
  import {
    Avatar,
    AvatarFallback,
    AvatarImage,
  } from '$lib/components/ui/avatar/index.js'
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js'
  import { useAuth } from '$lib/context/auth.svelte.js'
  import { LogOut, User } from '@lucide/svelte'

  const auth = useAuth()
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger
    class="focus-visible:ring-ring rounded-full focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none"
    aria-label="User menu"
  >
    <Avatar class="h-9 w-9 border">
      <AvatarImage src={null} alt="User avatar" />
      <AvatarFallback>
        <User class="h-4 w-4" />
      </AvatarFallback>
    </Avatar>
  </DropdownMenu.Trigger>

  <DropdownMenu.Content class="w-56" align="end" sideOffset={4}>
    <DropdownMenu.Label class="px-2 py-1.5 text-sm font-semibold">
      我的账户
    </DropdownMenu.Label>

    <DropdownMenu.Separator />

    <DropdownMenu.Group>
      <DropdownMenu.Label class="text-muted-foreground px-2 py-1.5 text-xs">
        登录身份
      </DropdownMenu.Label>
      <DropdownMenu.Item class="pointer-events-none px-2 py-1.5 text-sm">
        {auth.user?.username || 'User'}
      </DropdownMenu.Item>
    </DropdownMenu.Group>

    <DropdownMenu.Separator />

    <DropdownMenu.Item
      variant="destructive"
      onclick={async () => {
        await auth.logout()
        goto('/')
      }}
    >
      <LogOut class="mr-2 h-4 w-4" />
      退出登录
    </DropdownMenu.Item>
  </DropdownMenu.Content>
</DropdownMenu.Root>
