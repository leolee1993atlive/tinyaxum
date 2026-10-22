<script lang="ts">
  import { onMount } from 'svelte'
  import { goto } from '$app/navigation'
  import { page } from '$app/state'
  import { commonApi } from '$lib/api/common.js'
  import { datasourcesApi } from '$lib/api/datasources.js'
  import { sqlQueriesApi } from '$lib/api/sqlqueries.js'
  import type {
    DatasourceListVo,
    ModuleOptionVo,
    SqlQueryListVo,
  } from '$lib/api/types.js'
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte'
  import {
    CrudTableShell,
    DatasourceFilter,
    ModuleCategoryFilter,
  } from '$lib/components/crud/index.js'
  import Main from '$lib/components/layout/Main.svelte'
  import { Button } from '$lib/components/ui/button/index.js'
  import { useCrudTable } from '$lib/composables/use-crud-table.svelte.js'
  import { Copy, Pencil, Play, Plus, Trash2 } from '@lucide/svelte'
  import type { ColumnDef } from '@tanstack/table-core'
  import { toast } from 'svelte-sonner'
  import { useSQLQuery } from './sqlquery-state.svelte'

  const queryState = useSQLQuery()

  type EditorDrawerType = typeof import('./SqlQueryEditorDrawer.svelte').default
  let EditorDrawer: EditorDrawerType | null = $state(null)

  const SQL_QUERY_SORT_IDS = new Set([
    'sqlQueryId',
    'sqlQueryName',
    'dataSourceId',
    'createdDate',
  ])

  let datasources = $state<DatasourceListVo[]>([])
  let isDeleting = $state(false)
  let selectedDataSourceId = $state<number | null>(null)
  let moduleOptions = $state<ModuleOptionVo[]>([])
  let selectedModule = $state<string | null>(null)
  let selectedCategory = $state<string | null>(null)

  onMount(async () => {
    try {
      const pageResponse = await datasourcesApi.page({
        pageNo: 1,
        pageSize: 200,
      })
      datasources = pageResponse.data?.records ?? []
    } catch {
      datasources = []
    }
    try {
      const res = await commonApi.getModuleOptions()
      moduleOptions = res.data?.modules ?? []
    } catch {
      moduleOptions = []
    }
    const mod = await import('./SqlQueryEditorDrawer.svelte')
    EditorDrawer = mod.default

    // Open the editor directly when navigated to with ?edit=<id> (e.g. from
    // dashboard chart cards), then strip the param so refresh/back is clean.
    const editParam = page.url.searchParams.get('edit')
    if (editParam && /^\d+$/.test(editParam)) {
      queryState.openEditor(Number(editParam))
      const sp = new URLSearchParams(page.url.searchParams)
      sp.delete('edit')
      const qs = sp.toString()
      void goto(`${page.url.pathname}${qs ? `?${qs}` : ''}`, {
        replaceState: true,
        keepFocus: true,
        noScroll: true,
      })
    }
  })

  async function fetchPage(params: {
    pageNo: number
    pageSize: number
    keyword?: string
    sort?: string
    order?: 'asc' | 'desc'
  }) {
    await queryState.loadQueries({
      ...params,
      dataSourceId: selectedDataSourceId ?? undefined,
      module: selectedModule ?? undefined,
      category: selectedCategory ?? undefined,
    })
    return {
      rows: queryState.queries,
      totalRowCount: queryState.totalRowCount,
    }
  }

  const columns: ColumnDef<SqlQueryListVo>[] = [
    { accessorKey: 'sqlQueryName', header: '查询名称' },
    {
      accessorKey: 'sqlQueryDesc',
      header: '描述',
      meta: { cellClass: 'max-w-xs truncate' },
    },
    { accessorKey: 'dataSourceId', header: '数据源' },
    { accessorKey: 'module', header: '功能模块' },
    {
      id: 'actions',
      header: '操作',
      cell: (info) => info.row.original,
    },
  ]

  const crudTable = useCrudTable<SqlQueryListVo>({
    columns,
    sortableIds: SQL_QUERY_SORT_IDS,
    fetchPage,
  })

  async function handleRunQuery(query: SqlQueryListVo) {
    try {
      await sqlQueriesApi.run(query.sqlQueryId)
      toast.success(`查询 "${query.sqlQueryName}" 执行成功`)
    } catch (e) {
      const msg = e instanceof Error ? e.message : '执行查询失败'
      toast.error(msg)
    }
  }

  async function handleDelete() {
    if (!queryState.currentRow) return

    isDeleting = true
    try {
      const success = await queryState.deleteQuery(
        queryState.currentRow.sqlQueryId
      )
      if (success) {
        toast.success(`查询 "${queryState.currentRow.sqlQueryName}" 删除成功`)
        queryState.closeDialogs()
        crudTable.reload()
      }
    } finally {
      isDeleting = false
    }
  }

  function handleDatasourceChange(id: number | null) {
    selectedDataSourceId = id
    crudTable.resetPagination()
    crudTable.reload()
  }

  function handleModuleCategoryChange() {
    crudTable.resetPagination()
    crudTable.reload()
  }

  function handleDeleteOpenChange(open: boolean) {
    if (!open) {
      queryState.closeDialogs()
    }
  }
</script>

<Main class="flex flex-1 flex-col gap-4 sm:gap-6">
  <div class="flex flex-wrap items-end justify-between gap-2">
    <div>
      <h2 class="text-2xl font-bold tracking-tight">SQL 查询</h2>
      <p class="text-muted-foreground">在您的数据源上管理和执行 SQL 查询。</p>
    </div>
    <Button onclick={() => queryState.openEditor(null)}>
      <Plus class="mr-2 h-4 w-4" />
      添加查询
    </Button>
  </div>

  <CrudTableShell
    table={crudTable.table}
    isLoading={crudTable.isLoading}
    error={crudTable.error}
    columnCount={columns.length}
    searchPlaceholder="搜索查询..."
    emptyMessage="无结果。"
    customCellIds={new Set(['dataSourceId'])}
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
      <ModuleCategoryFilter
        {moduleOptions}
        bind:module={selectedModule}
        bind:category={selectedCategory}
        onChange={handleModuleCategoryChange}
      />
    {/snippet}
    {#snippet cellRenderer({ row, columnId })}
      {@const query = row as SqlQueryListVo}
      {#if columnId === 'dataSourceId'}
        {datasources.find((d) => d.dataSourceId === query.dataSourceId)?.name ??
          query.dataSourceId}
      {:else}
        <div class="flex items-center gap-1">
          <Button
            variant="ghost"
            size="icon"
            title="执行"
            aria-label="执行"
            onclick={() => handleRunQuery(query)}
          >
            <Play class="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            title="复制"
            aria-label="复制"
            onclick={() => queryState.openEditor(null, query.sqlQueryId)}
          >
            <Copy class="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            title="编辑"
            aria-label="编辑"
            onclick={() => queryState.openEditor(query.sqlQueryId)}
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
              queryState.setCurrentRow(query)
              queryState.setOpen('delete')
            }}
          >
            <Trash2 class="h-4 w-4" />
          </Button>
        </div>
      {/if}
    {/snippet}
  </CrudTableShell>
</Main>

{#if queryState.currentRow}
  <ConfirmDialog
    destructive
    open={queryState.open === 'delete'}
    onOpenChange={handleDeleteOpenChange}
    handleConfirm={handleDelete}
    title={`删除查询 "${queryState.currentRow.sqlQueryName}"？`}
    confirmText="删除"
    isLoading={isDeleting}
    class="max-w-md"
  >
    {#snippet desc()}
      这将软删除查询
      <strong>{queryState.currentRow?.sqlQueryName}</strong>。
    {/snippet}
  </ConfirmDialog>
{/if}

{#if EditorDrawer}
  <EditorDrawer dataSourceId={selectedDataSourceId ?? 0} {moduleOptions} />
{/if}
