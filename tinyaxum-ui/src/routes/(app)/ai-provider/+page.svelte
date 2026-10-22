<script lang="ts">
  import ProfileDropdown from '$lib/components/ProfileDropdown.svelte'
  import Search from '$lib/components/Search.svelte'
  import Header from '$lib/components/layout/Header.svelte'
  import AiProviderMutateDrawer from './components/AiProviderMutateDrawer.svelte'
  import AiProviderProvider from './components/AiProviderProvider.svelte'
  import AiProviderView from './components/AiProviderView.svelte'
</script>

<Header>
  <Search />
  <ProfileDropdown />
</Header>

<AiProviderProvider>
  <AiProviderMutateDrawer />
  <AiProviderView />
</AiProviderProvider>
