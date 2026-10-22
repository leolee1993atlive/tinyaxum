<script lang="ts">
  import ProfileDropdown from '$lib/components/ProfileDropdown.svelte'
  import Search from '$lib/components/Search.svelte'
  import Header from '$lib/components/layout/Header.svelte'
  import Main from '$lib/components/layout/Main.svelte'
  import * as Card from '$lib/components/ui/card/index.js'
  import { Construction } from '@lucide/svelte'
  import ConfigDrawer from '../dashboard/components/ConfigDrawer.svelte'
</script>

<Header>
  <Search />
  <ConfigDrawer />
  <ProfileDropdown />
</Header>

<Main class="flex flex-1 items-center justify-center">
  <Card.Root class="w-full max-w-md">
    <Card.Header class="items-center text-center">
      <Construction class="text-muted-foreground h-16 w-16" />
      <Card.Title class="mt-4 text-2xl">应用</Card.Title>
      <Card.Description>开发中</Card.Description>
    </Card.Header>
  </Card.Root>
</Main>
