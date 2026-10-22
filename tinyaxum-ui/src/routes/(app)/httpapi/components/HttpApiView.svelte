<script lang="ts">
  import { onMount, untrack } from 'svelte'
  import { commonApi } from '$lib/api/common.js'
  import type { ModuleOptionVo } from '$lib/api/types.js'
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte'
  import {
    CrudTableShell,
    ModuleCategoryFilter,
  } from '$lib/components/crud/index.js'
  import Main from '$lib/components/layout/Main.svelte'
  import { Badge } from '$lib/components/ui/badge/index.js'
  import { Button } from '$lib/components/ui/button/index.js'
  import * as Select from '$lib/components/ui/select/index.js'
  import { useCrudTable } from '$lib/composables/use-crud-table.svelte.js'
  import type { HttpApi } from '$lib/data/httpapi/schema.js'
  import { copyText } from '$lib/utils/clipboard.js'
  import {
    Copy,
    ExternalLink,
    Link,
    Pencil,
    Plus,
    Power,
    Trash2,
  } from '@lucide/svelte'
  import type { ColumnDef } from '@tanstack/table-core'
  import { toast } from 'svelte-sonner'
  import HttpApiMutateDrawer from './HttpApiMutateDrawer.svelte'
  import { useHttpApi } from './httpapi-state.svelte'

  const apiState = useHttpApi()

  const SORT_IDS = new Set([
    'httpApiName',
    'httpApiPath',
    'httpApiDesc',
    'createdDate',
  ])

  let isDeleting = $state(false)
  let deleteTarget: HttpApi | null = $state(null)
  let moduleOptions = $state<ModuleOptionVo[]>([])
  let selectedModule = $state<string | null>(null)
  let selectedCategory = $state<string | null>(null)

  onMount(async () => {
    try {
      const res = await commonApi.getModuleOptions()
      if (res.data?.modules) {
        moduleOptions = res.data.modules
      }
    } catch {
      moduleOptions = []
    }
  })

  type EnabledFilter = 'all' | 'enabled' | 'disabled'
  let enabledFilter = $state<EnabledFilter>('all')

  function enabledFilterValue(filter: EnabledFilter): number | undefined {
    if (filter === 'enabled') return 1
    if (filter === 'disabled') return 0
    return undefined
  }

  const enabledFilterLabel = $derived.by(() => {
    switch (enabledFilter) {
      case 'all':
        return '全部'
      case 'enabled':
        return '启用'
      case 'disabled':
        return '禁用'
    }
  })

  async function fetchPage(params: {
    pageNo: number
    pageSize: number
    keyword?: string
    sort?: string
    order?: 'asc' | 'desc'
  }) {
    const pageResult = await apiState.loadPage({
      pageNo: params.pageNo,
      pageSize: params.pageSize,
      keyword: params.keyword,
      enabled: enabledFilterValue(enabledFilter),
      module: selectedModule ?? undefined,
      category: selectedCategory ?? undefined,
    })
    return {
      rows: pageResult?.records ?? [],
      totalRowCount: pageResult?.total ?? 0,
    }
  }

  const columns: ColumnDef<HttpApi>[] = [
    { accessorKey: 'httpApiName', header: 'API名称' },
    { accessorKey: 'httpApiPath', header: '访问路径' },
    { accessorKey: 'module', header: '功能模块' },
    { accessorKey: 'httpApiDesc', header: '描述' },
    { accessorKey: 'enabled', header: '状态' },
    { accessorKey: 'createdDate', header: '创建时间' },
    {
      id: 'actions',
      header: '操作',
      cell: (info) => info.row.original,
    },
  ]

  const crudTable = useCrudTable<HttpApi>({
    columns,
    sortableIds: SORT_IDS,
    fetchPage,
  })

  $effect(() => {
    void apiState.listRefetchTick
    untrack(() => {
      if (apiState.listRefetchTick > 0) {
        crudTable.reload()
      }
    })
  })

  function handleModuleCategoryChange() {
    crudTable.resetPagination()
    crudTable.reload()
  }

  async function handleDelete() {
    if (!deleteTarget) return

    isDeleting = true
    try {
      const success = await apiState.deleteApi(deleteTarget.httpApiId)
      if (success) {
        crudTable.reload()
        deleteTarget = null
      }
    } finally {
      isDeleting = false
    }
  }

  function handleDeleteOpenChange(open: boolean) {
    if (!open) {
      deleteTarget = null
    }
  }

  async function handleToggleEnabled(row: HttpApi) {
    const success = await apiState.toggleEnabled(
      row.httpApiId,
      row.enabled !== 1
    )
    if (success) {
      crudTable.reload()
    }
  }

  function buildPublicUrl(api: HttpApi): string {
    return `${window.location.origin}/api/public/${api.httpApiPath}`
  }

  function handleCopyUrl(api: HttpApi) {
    const url = buildPublicUrl(api)
    void copyText(url).then(() => {
      toast.success('已复制: ' + url)
    })
  }

  function handleTestApi(api: HttpApi) {
    const url = buildPublicUrl(api)
    window.open(url, '_blank')
  }
</script>

<Main class="flex flex-1 flex-col gap-4 sm:gap-6">
  <div class="flex flex-wrap items-end justify-between gap-2">
    <div>
      <h2 class="text-2xl font-bold tracking-tight">HTTP API 列表</h2>
      <p class="text-muted-foreground">在此处管理您的 HTTP API 接口。</p>
    </div>
    <Button
      onclick={() => {
        apiState.setCurrentRow(null)
        apiState.setOpen('create')
      }}
    >
      <Plus class="mr-2 h-4 w-4" />
      创建 API
    </Button>
  </div>

  <CrudTableShell
    table={crudTable.table}
    isLoading={crudTable.isLoading}
    error={crudTable.error}
    columnCount={columns.length}
    searchPlaceholder="搜索 API..."
    emptyMessage="暂无 API 数据。"
    customCellIds={new Set(['enabled'])}
    bind:globalFilter={crudTable.globalFilter}
    onSearchInput={crudTable.resetPagination}
    onRefresh={() => crudTable.reload()}
  >
    {#snippet extraToolbar()}
      <ModuleCategoryFilter
        {moduleOptions}
        bind:module={selectedModule}
        bind:category={selectedCategory}
        onChange={handleModuleCategoryChange}
      />
      <Select.Root
        type="single"
        value={enabledFilter}
        onValueChange={(value) => {
          enabledFilter = value as EnabledFilter
          crudTable.resetPagination()
        }}
      >
        <Select.Trigger class="w-[140px]">
          <span>{enabledFilterLabel}</span>
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="all" label="全部">全部</Select.Item>
          <Select.Item value="enabled" label="启用">启用</Select.Item>
          <Select.Item value="disabled" label="禁用">禁用</Select.Item>
        </Select.Content>
      </Select.Root>
    {/snippet}

    {#snippet cellRenderer({ row, columnId })}
      {#if columnId === 'enabled'}
        {@const api = row as HttpApi}
        {#if api.enabled === 1}
          <Badge>启用</Badge>
        {:else}
          <Badge variant="secondary">禁用</Badge>
        {/if}
      {:else}
        {@const api = row as HttpApi}
        <div class="flex items-center gap-1">
          <Button
            variant="ghost"
            size="icon"
            title="复制URL"
            aria-label="复制URL"
            onclick={() => handleCopyUrl(api)}
          >
            <Link class="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            title="测试"
            aria-label="测试"
            onclick={() => handleTestApi(api)}
          >
            <ExternalLink class="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            title="复制"
            aria-label="复制"
            onclick={() => {
              void apiState.copyApi(api.httpApiId)
            }}
          >
            <Copy class="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            title="编辑"
            aria-label="编辑"
            onclick={() => {
              apiState.setCurrentRow(api)
              apiState.setOpen('edit')
            }}
          >
            <Pencil class="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            title={api.enabled === 1 ? '禁用' : '启用'}
            aria-label={api.enabled === 1 ? '禁用' : '启用'}
            onclick={() => handleToggleEnabled(api)}
          >
            <Power class="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            class="text-destructive hover:text-destructive"
            title="删除"
            aria-label="删除"
            onclick={() => {
              deleteTarget = api
            }}
          >
            <Trash2 class="h-4 w-4" />
          </Button>
        </div>
      {/if}
    {/snippet}
  </CrudTableShell>
</Main>

<HttpApiMutateDrawer />

{#if deleteTarget}
  <ConfirmDialog
    destructive
    open={!!deleteTarget}
    onOpenChange={handleDeleteOpenChange}
    handleConfirm={handleDelete}
    title={`删除 API "${deleteTarget.httpApiName}"？`}
    confirmText="删除"
    isLoading={isDeleting}
    class="max-w-md"
  >
    {#snippet desc()}
      这将删除 HTTP API
      <strong>{(deleteTarget as HttpApi)?.httpApiName}</strong>。
      删除后无法恢复。
    {/snippet}
  </ConfirmDialog>
{/if}
