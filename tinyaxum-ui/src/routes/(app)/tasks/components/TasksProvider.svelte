<script lang="ts">
  import { onMount, type Snippet } from 'svelte'
  import { TasksState, provideTasks } from './tasks-state.svelte'

  let { children }: { children: Snippet } = $props()

  const tasksState = new TasksState()
  provideTasks(tasksState)

  onMount(() => {
    void tasksState.loadDatasources()
  })
</script>

{@render children()}
