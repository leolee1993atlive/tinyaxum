<script lang="ts">
  import type { SqlSnippetListVo } from '$lib/api/types.js'
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte'
  import { CrudTableShell } from '$lib/components/crud/index.js'
  import Main from '$lib/components/layout/Main.svelte'
  import SqlSnippetFormDrawer from '$lib/components/sql-snippet/sql-snippet-form-drawer.svelte'
  import { Button } from '$lib/components/ui/button/index.js'
  import { useCrudTable } from '$lib/composables/use-crud-table.svelte.js'
  import { copyText } from '$lib/utils/clipboard.js'
  import { Pencil, Plus, Trash2, Copy, Check } from '@lucide/svelte'
  import type { ColumnDef } from '@tanstack/table-core'
  import { toast } from 'svelte-sonner'
  import { useSqlSnippets } from './sqlsnippet-state.svelte'

  const snippetState = useSqlSnippets()

  const SQL_SNIPPET_SORT_IDS = new Set([
    'sqlSnippetId',
    'sqlSnippetName',
    'createdDate',
  ])

  let copiedId = $state<number | null>(null)
  let isDeleting = $state(false)

  async function fetchPage(params: {
    pageNo: number
    pageSize: number
    keyword?: string
    sort?: string
    order?: 'asc' | 'desc'
  }) {
    await snippetState.loadSnippets(params)
    return {
      rows: snippetState.snippets,
      totalRowCount: snippetState.totalRowCount,
    }
  }

  const columns: ColumnDef<SqlSnippetListVo>[] = [
    { accessorKey: 'sqlSnippetName', header: '名称' },
    { accessorKey: 'sqlSnippetDesc', header: '描述' },
    { accessorKey: 'tags', header: '标签' },
    {
      id: 'actions',
      header: '操作',
      cell: (info) => info.row.original,
    },
  ]

  const crudTable = useCrudTable<SqlSnippetListVo>({
    columns,
    sortableIds: SQL_SNIPPET_SORT_IDS,
    fetchPage,
  })

  async function copyToClipboard(content: string, id: number) {
    try {
      await copyText(content)
      copiedId = id
      toast.success('片段已复制到剪贴板')
      setTimeout(() => {
        copiedId = null
      }, 2000)
    } catch {
      toast.error('复制到剪贴板失败')
    }
  }

  async function handleDelete() {
    if (!snippetState.currentRow) return

    isDeleting = true
    try {
      const success = await snippetState.deleteSnippet(
        snippetState.currentRow.sqlSnippetId
      )
      if (success) {
        toast.success(
          `片段 "${snippetState.currentRow.sqlSnippetName}" 删除成功`
        )
        snippetState.closeDialogs()
        crudTable.reload()
      }
    } finally {
      isDeleting = false
    }
  }

  function handleDeleteOpenChange(open: boolean) {
    if (!open) {
      snippetState.closeDialogs()
    }
  }

  function handleFormOpenChange(open: boolean) {
    if (!open) {
      snippetState.setOpen(null)
      snippetState.resetCopySnippet()
      setTimeout(() => {
        snippetState.setCurrentRow(null)
      }, 200)
    }
  }
</script>

<Main class="flex flex-1 flex-col gap-4 sm:gap-6">
  <div class="flex flex-wrap items-end justify-between gap-2">
    <div>
      <h2 class="text-2xl font-bold tracking-tight">SQL 片段</h2>
      <p class="text-muted-foreground">管理可重用的 SQL 片段和代码模板。</p>
    </div>
    <Button
      onclick={() => {
        snippetState.setCurrentRow(null)
        snippetState.setOpen('create')
      }}
    >
      <Plus class="mr-2 h-4 w-4" />
      添加片段
    </Button>
  </div>

  <CrudTableShell
    table={crudTable.table}
    isLoading={crudTable.isLoading}
    error={crudTable.error}
    columnCount={columns.length}
    searchPlaceholder="搜索片段..."
    emptyMessage="无结果。"
    bind:globalFilter={crudTable.globalFilter}
    onSearchInput={crudTable.resetPagination}
    onRefresh={() => crudTable.reload()}
  >
    {#snippet cellRenderer({ row })}
      {@const snippet = row as SqlSnippetListVo}
      <div class="flex items-center gap-1">
        <Button
          variant="ghost"
          size="icon"
          title="复制到剪贴板"
          aria-label="复制到剪贴板"
          onclick={() =>
            copyToClipboard(snippet.sqlSnippetContent, snippet.sqlSnippetId)}
        >
          {#if copiedId === snippet.sqlSnippetId}
            <Check class="h-4 w-4 text-green-500" />
          {:else}
            <Copy class="h-4 w-4" />
          {/if}
        </Button>
        <Button
          variant="ghost"
          size="icon"
          title="复制片段"
          aria-label="复制片段"
          onclick={async () => {
            const ok = await snippetState.copySnippet(snippet.sqlSnippetId)
            if (!ok && snippetState.error) {
              toast.error(snippetState.error)
            }
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
            snippetState.setCurrentRow(snippet)
            snippetState.setOpen('update')
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
            snippetState.setCurrentRow(snippet)
            snippetState.setOpen('delete')
          }}
        >
          <Trash2 class="h-4 w-4" />
        </Button>
      </div>
    {/snippet}
  </CrudTableShell>
</Main>

<SqlSnippetFormDrawer
  open={snippetState.open === 'create' ||
    snippetState.open === 'update' ||
    snippetState.open === 'copy'}
  mode={snippetState.open === 'update'
    ? 'update'
    : snippetState.open === 'copy'
      ? 'copy'
      : 'create'}
  initialSnippet={snippetState.open === 'update'
    ? snippetState.currentRow
    : snippetState.open === 'copy'
      ? snippetState.copySnippetData
      : null}
  onOpenChange={handleFormOpenChange}
  onsuccess={() => crudTable.reload()}
/>

{#if snippetState.currentRow}
  <ConfirmDialog
    destructive
    open={snippetState.open === 'delete'}
    onOpenChange={handleDeleteOpenChange}
    handleConfirm={handleDelete}
    title={`删除片段 "${snippetState.currentRow.sqlSnippetName}"？`}
    confirmText="删除"
    isLoading={isDeleting}
    class="max-w-md"
  >
    {#snippet desc()}
      这将软删除片段
      <strong>{snippetState.currentRow?.sqlSnippetName}</strong>。
    {/snippet}
  </ConfirmDialog>
{/if}
