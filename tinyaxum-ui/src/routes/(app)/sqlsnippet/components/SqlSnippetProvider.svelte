<script lang="ts">
  import type { Snippet } from 'svelte'
  import {
    SqlSnippetState,
    provideSqlSnippets,
  } from './sqlsnippet-state.svelte'

  let { children }: { children: Snippet } = $props()

  const snippetState = new SqlSnippetState()
  provideSqlSnippets(snippetState)
</script>

{@render children()}
