<script lang="ts">
  import { onMount, untrack } from 'svelte'
  import { transtaskApi } from '$lib/api/transtask.js'
  import {
    CrudTableShell,
    DatasourceFilter,
  } from '$lib/components/crud/index.js'
  import { Badge } from '$lib/components/ui/badge/index.js'
  import { Button } from '$lib/components/ui/button/index.js'
  import { useCrudTable } from '$lib/composables/use-crud-table.svelte.js'
  import {
    Copy,
    LoaderCircle,
    Pause,
    Pencil,
    Play,
    SquareActivity,
    Trash2,
  } from '@lucide/svelte'
  import type { ColumnDef } from '@tanstack/table-core'
  import { toast } from 'svelte-sonner'
  import type { TransTask } from '../data/schema.js'
  import { useTasks } from './tasks-state.svelte'

  const tasksState = useTasks()

  const datasources = $derived(tasksState.datasources)
  let selectedDataSourceId = $state<number | null>(null)
  /** 每分钟更新，驱动相对时间列的重新渲染 */
  let now = $state(Date.now())

  onMount(() => {
    // 相对时间每分钟刷新一次
    const clockId = setInterval(() => {
      now = Date.now()
    }, 60_000)

    tasksState.startEventStream()

    return () => {
      clearInterval(clockId)
      tasksState.stopEventStream()
    }
  })

  // ── 相对时间 ───────────────────────────────────────────────────────────────
  function relativeTime(dateStr: string | null | undefined): string {
    if (!dateStr) return '—'
    const date = new Date(dateStr)
    if (isNaN(date.getTime())) return '—'
    const diffSec = Math.floor((now - date.getTime()) / 1000)
    if (diffSec < 5) return '刚刚'
    if (diffSec < 60) return `${diffSec}秒前`
    const diffMin = Math.floor(diffSec / 60)
    if (diffMin < 60) return `${diffMin}分钟前`
    const diffHour = Math.floor(diffMin / 60)
    if (diffHour < 24) return `${diffHour}小时前`
    const diffDay = Math.floor(diffHour / 24)
    if (diffDay < 30) return `${diffDay}天前`
    const diffMonth = Math.floor(diffDay / 30)
    if (diffMonth < 12) return `${diffMonth}个月前`
    return `${Math.floor(diffMonth / 12)}年前`
  }

  const TASK_SORT_IDS = new Set([
    'transTaskId',
    'dataSourceId',
    'tableName',
    'taskStatus',
    'createdDate',
  ])

  async function fetchPage(params: {
    pageNo: number
    pageSize: number
    keyword?: string
    sort?: string
    order?: 'asc' | 'desc'
  }) {
    await tasksState.loadTasks({
      ...params,
      dataSourceId: selectedDataSourceId ?? undefined,
    })
    return {
      rows: tasksState.tasks,
      totalRowCount: tasksState.totalRowCount,
    }
  }

  const columns: ColumnDef<TransTask>[] = [
    { accessorKey: 'transTaskId', header: '任务ID' },
    { accessorKey: 'dataSourceId', header: '数据源' },
    { accessorKey: 'tableName', header: '表名' },
    { accessorKey: 'tableComment', header: '注释' },
    { accessorKey: 'taskCron', header: 'Cron表达式' },
    { accessorKey: 'taskStatus', header: '状态' },
    { accessorKey: 'rowCount', header: '行数' },
    { accessorKey: 'lastTransTime', header: '上次传输' },
    {
      id: 'actions',
      header: '操作',
      cell: (info) => info.row.original,
    },
  ]

  const crudTable = useCrudTable<TransTask>({
    columns,
    sortableIds: TASK_SORT_IDS,
    fetchPage,
  })

  $effect(() => {
    void tasksState.listRefetchTick
    untrack(() => {
      if (tasksState.listRefetchTick > 0) {
        crudTable.reload()
      }
    })
  })

  function handleTest(task: TransTask) {
    tasksState.setCurrentRow(task)
    tasksState.setOpen('syncProgress')
    if (!tasksState.runningTaskIds.has(task.transTaskId)) {
      void tasksState.startTaskSync(task.transTaskId)
    }
  }

  async function handleEnable(task: TransTask) {
    const success = await transtaskApi.start(task.transTaskId)
    if (success) {
      toast.success(`任务 ${task.transTaskId} 已启用定时调度`)
      tasksState.requestListRefetch()
    } else {
      toast.error(`任务 ${task.transTaskId} 启用失败`)
    }
  }

  async function handleDisable(task: TransTask) {
    const success = await transtaskApi.stop(task.transTaskId)
    if (success) {
      toast.success(`任务 ${task.transTaskId} 已停用定时调度`)
      tasksState.requestListRefetch()
    } else {
      toast.error(`任务 ${task.transTaskId} 停用失败`)
    }
  }

  function handleDatasourceChange(id: number | null) {
    selectedDataSourceId = id
    crudTable.resetPagination()
    crudTable.reload()
  }

  async function handleCopy(task: TransTask) {
    const ok = await tasksState.copyTask(task.transTaskId)
    if (!ok && tasksState.error) {
      toast.error(tasksState.error)
    }
  }
</script>

<CrudTableShell
  table={crudTable.table}
  isLoading={crudTable.isLoading}
  error={crudTable.error}
  columnCount={columns.length}
  searchPlaceholder="搜索任务..."
  emptyMessage="未找到任务。"
  customCellIds={new Set(['dataSourceId', 'taskStatus', 'lastTransTime'])}
  bind:globalFilter={crudTable.globalFilter}
  onSearchInput={crudTable.resetPagination}
  onRefresh={() => crudTable.reload()}
>
  {#snippet extraToolbar()}
    <DatasourceFilter
      value={selectedDataSourceId}
      {datasources}
      onChange={handleDatasourceChange}
    />
  {/snippet}
  {#snippet cellRenderer({ row, columnId })}
    {@const task = row as TransTask}

    {#if columnId === 'dataSourceId'}
      {datasources.find((d) => d.dataSourceId === task.dataSourceId)?.name ??
        task.dataSourceId}
    {:else if columnId === 'taskStatus'}
      {@const isSyncing =
        task.isSyncing === 1 || tasksState.runningTaskIds.has(task.transTaskId)}
      {@const isEnabled = task.taskStatus.toLowerCase() === 'start'}
      {#if isSyncing}
        <Badge
          class="gap-1 border-blue-200 bg-blue-500/10 text-blue-600 dark:border-blue-800 dark:text-blue-400"
        >
          <LoaderCircle class="size-3 animate-spin" />
          同步中
        </Badge>
      {:else if isEnabled}
        <Badge
          class="border-green-200 bg-green-500/10 text-green-700 dark:border-green-800 dark:text-green-400"
        >
          已启用
        </Badge>
      {:else}
        <Badge variant="outline" class="text-muted-foreground">未启用</Badge>
      {/if}
    {:else if columnId === 'lastTransTime'}
      <span class="text-muted-foreground text-sm" title={task.lastTransTime}>
        {relativeTime(task.lastTransTime)}
      </span>
    {:else}
      <!-- actions 列 -->
      {@const isSyncing =
        task.isSyncing === 1 || tasksState.runningTaskIds.has(task.transTaskId)}
      {@const isEnabled = task.taskStatus.toLowerCase() === 'start'}
      <div class="flex items-center gap-1">
        <Button
          variant="ghost"
          size="icon-sm"
          title={isSyncing ? '查看同步进度' : '立即同步'}
          aria-label={isSyncing ? '查看同步进度' : '立即同步'}
          onclick={() => handleTest(task)}
        >
          {#if isSyncing}
            <LoaderCircle class="size-4 animate-spin text-blue-500" />
          {:else}
            <SquareActivity class="size-4" />
          {/if}
        </Button>

        {#if isEnabled}
          <Button
            variant="ghost"
            size="icon-sm"
            class="text-amber-500 hover:bg-amber-500/10 hover:text-amber-600"
            title="停用定时调度"
            aria-label="停用定时调度"
            onclick={() => handleDisable(task)}
          >
            <Pause class="size-4" />
          </Button>
        {:else}
          <Button
            variant="ghost"
            size="icon-sm"
            class="text-green-600 hover:bg-green-500/10 hover:text-green-700 dark:text-green-500 dark:hover:text-green-400"
            title="启用定时调度"
            aria-label="启用定时调度"
            onclick={() => handleEnable(task)}
          >
            <Play class="size-4" />
          </Button>
        {/if}

        <Button
          variant="ghost"
          size="icon-sm"
          title="复制"
          aria-label="复制"
          disabled={tasksState.copyingId === task.transTaskId}
          onclick={() => handleCopy(task)}
        >
          {#if tasksState.copyingId === task.transTaskId}
            <LoaderCircle class="size-4 animate-spin" />
          {:else}
            <Copy class="size-4" />
          {/if}
        </Button>

        <Button
          variant="ghost"
          size="icon-sm"
          title="编辑"
          aria-label="编辑"
          onclick={() => {
            tasksState.setCurrentRow(task)
            tasksState.setOpen('update')
          }}
        >
          <Pencil class="size-4" />
        </Button>
        <Button
          variant="ghost"
          size="icon-sm"
          class="text-destructive hover:bg-destructive/10 hover:text-destructive"
          title="删除"
          aria-label="删除"
          onclick={() => {
            tasksState.setCurrentRow(task)
            tasksState.setOpen('delete')
          }}
        >
          <Trash2 class="size-4" />
        </Button>
      </div>
    {/if}
  {/snippet}
</CrudTableShell>
