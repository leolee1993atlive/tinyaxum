<script lang="ts">
  import type { Snippet } from 'svelte'
  import { HttpApiState, provideHttpApi } from './httpapi-state.svelte'

  let { children }: { children: Snippet } = $props()

  const httpApiCtx = new HttpApiState()
  provideHttpApi(httpApiCtx)
</script>

{@render children()}
