<script lang="ts">
  import { Button } from '$lib/components/ui/button/index.js'
  import { Layers, Plus } from '@lucide/svelte'
  import { useTasks } from './tasks-state.svelte'

  const tasks = useTasks()
</script>

<div class="flex flex-wrap items-center gap-2">
  <Button onclick={() => tasks.setOpen('create')}>
    <Plus class="mr-2 h-4 w-4" />
    添加任务
  </Button>
  <Button variant="outline" onclick={() => tasks.setOpen('batchCreate')}>
    <Layers class="mr-2 h-4 w-4" />
    批量添加
  </Button>
</div>
