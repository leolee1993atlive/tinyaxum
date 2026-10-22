<script lang="ts">
  import { untrack } from 'svelte'
  import type { DatasourceListVo } from '$lib/api/types.js'
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte'
  import { CrudTableShell } from '$lib/components/crud/index.js'
  import Main from '$lib/components/layout/Main.svelte'
  import { Button } from '$lib/components/ui/button/index.js'
  import { useCrudTable } from '$lib/composables/use-crud-table.svelte.js'
  import {
    Pencil,
    SquareActivity,
    Plus,
    Trash2,
    LoaderCircle,
    RefreshCw,
    Copy,
  } from '@lucide/svelte'
  import type { ColumnDef } from '@tanstack/table-core'
  import { toast } from 'svelte-sonner'
  import DatasourceMutateDrawer from './DatasourceMutateDrawer.svelte'
  import { useDatasources } from './datasource-state.svelte'

  const datasourcesState = useDatasources()

  const DATASOURCE_SORT_IDS = new Set([
    'dataSourceId',
    'code',
    'name',
    'dbType',
    'createdDate',
  ])

  let isDeleting = $state(false)
  let testingId = $state<number | null>(null)
  let refreshingSchemaId = $state<number | null>(null)
  let copyingId = $state<number | null>(null)

  async function fetchPage(params: {
    pageNo: number
    pageSize: number
    keyword?: string
    sort?: string
    order?: 'asc' | 'desc'
  }) {
    await datasourcesState.loadDatasources(params)
    return {
      rows: datasourcesState.datasources,
      totalRowCount: datasourcesState.totalRowCount,
    }
  }

  const columns: ColumnDef<DatasourceListVo>[] = [
    { accessorKey: 'code', header: '编码' },
    { accessorKey: 'name', header: '名称' },
    { accessorKey: 'dbType', header: '数据库类型' },
    { accessorKey: 'dbName', header: '数据库名称' },
    {
      accessorKey: 'remark',
      header: '备注',
      cell: (info) => (info.getValue() as string | undefined) || '-',
    },
    {
      id: 'actions',
      header: '操作',
      cell: (info) => info.row.original,
    },
  ]

  const crudTable = useCrudTable<DatasourceListVo>({
    columns,
    sortableIds: DATASOURCE_SORT_IDS,
    fetchPage,
  })

  $effect(() => {
    void datasourcesState.listRefetchTick
    untrack(() => {
      if (datasourcesState.listRefetchTick > 0) {
        crudTable.reload()
      }
    })
  })

  async function handleTestConnection(datasource: DatasourceListVo) {
    testingId = datasource.dataSourceId
    try {
      const success = await datasourcesState.testConnection(
        datasource.dataSourceId
      )
      if (success) {
        toast.success(`数据源 "${datasource.code}" 连接成功`)
      } else {
        const m = datasourcesState.testResult?.message
        if (m) {
          toast.error(m)
        }
      }
    } finally {
      testingId = null
    }
  }

  async function handleRefreshSchema(datasource: DatasourceListVo) {
    refreshingSchemaId = datasource.dataSourceId
    try {
      const ok = await datasourcesState.refreshSchema(datasource.dataSourceId)
      if (ok) {
        toast.success(`已刷新数据源 "${datasource.code}" 的表/列元数据`)
      } else if (datasourcesState.error) {
        toast.error(datasourcesState.error)
      }
    } finally {
      refreshingSchemaId = null
    }
  }

  async function handleDeleteDatasource() {
    if (!datasourcesState.currentRow) return

    isDeleting = true
    try {
      const success = await datasourcesState.deleteDatasource(
        datasourcesState.currentRow.dataSourceId
      )
      if (success) {
        toast.success('数据源删除成功')
        datasourcesState.setOpen(null)
        datasourcesState.setCurrentRow(null)
        crudTable.reload()
      }
    } finally {
      isDeleting = false
    }
  }

  function handleDeleteOpenChange(open: boolean) {
    if (!open) {
      datasourcesState.setOpen(null)
      setTimeout(() => {
        datasourcesState.setCurrentRow(null)
      }, 200)
    }
  }
</script>

<Main class="flex flex-1 flex-col gap-4 sm:gap-6">
  <div class="flex flex-wrap items-end justify-between gap-2">
    <div>
      <h2 class="text-2xl font-bold tracking-tight">数据源列表</h2>
      <p class="text-muted-foreground">在此处管理您的数据库连接和数据源。</p>
    </div>
    <Button
      onclick={() => {
        datasourcesState.setCurrentRow(null)
        datasourcesState.setOpen('create')
      }}
    >
      <Plus class="mr-2 h-4 w-4" />
      添加数据源
    </Button>
  </div>

  <CrudTableShell
    table={crudTable.table}
    isLoading={crudTable.isLoading}
    error={crudTable.error}
    columnCount={columns.length}
    searchPlaceholder="搜索数据源..."
    emptyMessage="未找到数据源。"
    bind:globalFilter={crudTable.globalFilter}
    onSearchInput={crudTable.resetPagination}
    onRefresh={() => crudTable.reload()}
  >
    {#snippet cellRenderer({ row })}
      {@const datasource = row as DatasourceListVo}
      <div class="flex items-center gap-1">
        <Button
          variant="ghost"
          size="icon"
          title="测试"
          aria-label="测试"
          onclick={() => handleTestConnection(datasource)}
          disabled={testingId === datasource.dataSourceId}
        >
          {#if testingId === datasource.dataSourceId}
            <LoaderCircle class="h-4 w-4 animate-spin" />
          {:else}
            <SquareActivity class="size-4" />
          {/if}
        </Button>
        <Button
          variant="ghost"
          size="icon"
          title="刷新表/列元数据"
          aria-label="刷新元数据"
          onclick={() => handleRefreshSchema(datasource)}
          disabled={refreshingSchemaId === datasource.dataSourceId}
        >
          {#if refreshingSchemaId === datasource.dataSourceId}
            <LoaderCircle class="h-4 w-4 animate-spin" />
          {:else}
            <RefreshCw class="h-4 w-4" />
          {/if}
        </Button>
        <Button
          variant="ghost"
          size="icon"
          title="复制"
          aria-label="复制"
          onclick={async () => {
            copyingId = datasource.dataSourceId
            try {
              const ok = await datasourcesState.copyDatasource(
                datasource.dataSourceId
              )
              if (!ok && datasourcesState.error) {
                toast.error(datasourcesState.error)
              }
            } finally {
              copyingId = null
            }
          }}
          disabled={copyingId === datasource.dataSourceId}
        >
          {#if copyingId === datasource.dataSourceId}
            <LoaderCircle class="h-4 w-4 animate-spin" />
          {:else}
            <Copy class="h-4 w-4" />
          {/if}
        </Button>
        <Button
          variant="ghost"
          size="icon"
          title="编辑"
          aria-label="编辑"
          onclick={() => {
            datasourcesState.setCurrentRow(datasource)
            datasourcesState.setOpen('update')
          }}
        >
          <Pencil class="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          class="text-destructive hover:text-destructive"
          title="删除"
          aria-label="删除"
          onclick={() => {
            datasourcesState.setCurrentRow(datasource)
            datasourcesState.setOpen('delete')
          }}
        >
          <Trash2 class="h-4 w-4" />
        </Button>
      </div>
    {/snippet}
  </CrudTableShell>
</Main>

<DatasourceMutateDrawer />

{#if datasourcesState.currentRow}
  <ConfirmDialog
    destructive
    open={datasourcesState.open === 'delete'}
    onOpenChange={handleDeleteOpenChange}
    handleConfirm={handleDeleteDatasource}
    title={`删除数据源 "${datasourcesState.currentRow.code}"？`}
    confirmText="删除"
    isLoading={isDeleting}
    class="max-w-md"
  >
    {#snippet desc()}
      这将软删除数据源
      <strong>{datasourcesState.currentRow?.code}</strong>。
    {/snippet}
  </ConfirmDialog>
{/if}
