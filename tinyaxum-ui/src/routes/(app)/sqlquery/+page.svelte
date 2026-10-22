<script lang="ts">
  import ProfileDropdown from '$lib/components/ProfileDropdown.svelte'
  import Search from '$lib/components/Search.svelte'
  import Header from '$lib/components/layout/Header.svelte'
  import SQLQueryProvider from './components/SQLQueryProvider.svelte'
  import SQLQueryView from './components/SQLQueryView.svelte'
</script>

<SQLQueryProvider>
  <Header>
    <Search />
    <ProfileDropdown />
  </Header>

  <SQLQueryView />
</SQLQueryProvider>
