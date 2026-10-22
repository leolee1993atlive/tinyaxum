<script lang="ts">
  import ProfileDropdown from '$lib/components/ProfileDropdown.svelte'
  import Search from '$lib/components/Search.svelte'
  import Header from '$lib/components/layout/Header.svelte'
  import DatasourceProvider from './components/DatasourceProvider.svelte'
  import DatasourceView from './components/DatasourceView.svelte'
</script>

<Header>
  <Search />
  <ProfileDropdown />
</Header>

<DatasourceProvider>
  <DatasourceView />
</DatasourceProvider>
