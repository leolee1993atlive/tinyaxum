<script lang="ts">
  import ProfileDropdown from '$lib/components/ProfileDropdown.svelte'
  import Search from '$lib/components/Search.svelte'
  import Header from '$lib/components/layout/Header.svelte'
  import SqlSnippetProvider from './components/SqlSnippetProvider.svelte'
  import SqlSnippetView from './components/SqlSnippetView.svelte'
</script>

<Header>
  <Search />
  <ProfileDropdown />
</Header>

<SqlSnippetProvider>
  <SqlSnippetView />
</SqlSnippetProvider>
