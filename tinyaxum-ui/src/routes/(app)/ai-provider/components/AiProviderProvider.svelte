<script lang="ts">
  import type { Snippet } from 'svelte'
  import {
    AiProviderState,
    provideAiProvider,
  } from './ai-provider-state.svelte'

  let { children }: { children: Snippet } = $props()

  const ctx = new AiProviderState()
  provideAiProvider(ctx)
</script>

{@render children()}
