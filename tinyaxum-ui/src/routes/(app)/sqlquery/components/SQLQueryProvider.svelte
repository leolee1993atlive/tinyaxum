<script lang="ts">
  import type { Snippet } from 'svelte'
  import { SQLQueryState, provideSQLQuery } from './sqlquery-state.svelte'

  let { children }: { children: Snippet } = $props()

  const sqlQueryState = new SQLQueryState()
  provideSQLQuery(sqlQueryState)
</script>

{@render children()}
