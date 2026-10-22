<script lang="ts">
  import ProfileDropdown from '#lib/components/ProfileDropdown.svelte'
  import Search from '#lib/components/Search.svelte'
  import Header from '#lib/components/layout/Header.svelte'
  import UsersProvider from './components/UsersProvider.svelte'
  import UsersView from './components/UsersView.svelte'
</script>

<Header>
  <Search />
  <ProfileDropdown />
</Header>

<UsersProvider>
  <UsersView />
</UsersProvider>
