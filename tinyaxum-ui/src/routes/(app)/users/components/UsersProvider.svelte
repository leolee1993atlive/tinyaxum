<script lang="ts">
  import type { Snippet } from 'svelte'
  import { UsersState, provideUsers } from './users-state.svelte'

  let { children }: { children: Snippet } = $props()

  const usersCtx = new UsersState()
  provideUsers(usersCtx)
</script>

{@render children()}
