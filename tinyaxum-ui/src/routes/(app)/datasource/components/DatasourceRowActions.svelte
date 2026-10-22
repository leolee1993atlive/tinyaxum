<script lang="ts">
  import type { DatasourceListVo } from '$lib/api/types.js'
  import { Button } from '$lib/components/ui/button/index.js'
  import { Play, Pencil, Trash2 } from '@lucide/svelte'
  import { toast } from 'svelte-sonner'
  import { useDatasources } from './datasource-state.svelte'

  interface Props {
    row: DatasourceListVo
  }

  let { row }: Props = $props()

  const datasources = useDatasources()
</script>

<div class="flex items-center gap-1">
  <Button
    variant="ghost"
    size="sm"
    onclick={async () => {
      const success = await datasources.testConnection(row.dataSourceId)
      if (success) {
        toast.success('连接成功！')
      }
    }}
  >
    <Play class="h-4 w-4" />
    测试
  </Button>
  <Button
    variant="ghost"
    size="sm"
    onclick={() => {
      datasources.setCurrentRow(row)
      datasources.setOpen('update')
    }}
  >
    <Pencil class="h-4 w-4" />
    编辑
  </Button>
  <Button
    variant="ghost"
    size="sm"
    class="text-destructive hover:text-destructive"
    onclick={() => {
      datasources.setCurrentRow(row)
      datasources.setOpen('delete')
    }}
  >
    <Trash2 class="h-4 w-4" />
    删除
  </Button>
</div>
