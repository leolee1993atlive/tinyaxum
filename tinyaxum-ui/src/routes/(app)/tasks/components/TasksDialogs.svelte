<script lang="ts">
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte'
  import { toast } from 'svelte-sonner'
  import TaskBatchCreateDrawer from './TaskBatchCreateDrawer.svelte'
  import TaskMutateDrawer from './TaskMutateDrawer.svelte'
  import TaskSyncProgressDrawer from './TaskSyncProgressDrawer.svelte'
  import { useTasks } from './tasks-state.svelte'

  const tasks = useTasks()

  let isDeleting = $state(false)

  async function handleDeleteConfirm() {
    isDeleting = true
    try {
      const taskId = tasks.currentRow?.transTaskId
      const success = await tasks.deleteCurrentTask()
      if (success) {
        toast.success(`任务 "${taskId}" 删除成功`)
        tasks.setOpen(null)
        tasks.setCurrentRow(null)
      }
    } finally {
      isDeleting = false
    }
  }

  function handleOpenChange(open: boolean) {
    if (!open) {
      tasks.setOpen(null)
      setTimeout(() => {
        tasks.setCurrentRow(null)
      }, 200)
    }
  }
</script>

<TaskMutateDrawer />
<TaskBatchCreateDrawer />
<TaskSyncProgressDrawer />

{#if tasks.currentRow}
  <ConfirmDialog
    destructive
    open={tasks.open === 'delete'}
    onOpenChange={handleOpenChange}
    handleConfirm={handleDeleteConfirm}
    title={`删除任务 ${tasks.currentRow.transTaskId}？`}
    confirmText="删除"
    isLoading={isDeleting}
    class="max-w-md"
  >
    {#snippet desc()}
      这将软删除任务
      <strong>{tasks.currentRow?.transTaskId}</strong>。
    {/snippet}
  </ConfirmDialog>
{/if}
