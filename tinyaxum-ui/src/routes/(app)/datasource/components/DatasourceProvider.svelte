<script lang="ts">
  import type { Snippet } from 'svelte'
  import { DatasourceState, provideDatasource } from './datasource-state.svelte'

  let { children }: { children: Snippet } = $props()

  const datasourceCtx = new DatasourceState()
  provideDatasource(datasourceCtx)
</script>

{@render children()}
