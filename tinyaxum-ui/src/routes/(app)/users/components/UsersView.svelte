<script lang="ts">
  import { untrack } from 'svelte'
  import ConfirmDialog from '#lib/components/ConfirmDialog.svelte'
  import { CrudTableShell } from '#lib/components/crud/index.js'
  import Main from '#lib/components/layout/Main.svelte'
  import { Button } from '#lib/components/ui/button/index.js'
  import { useCrudTable } from '#lib/composables/use-crud-table.svelte.js'
  import type { User } from '#lib/data/users/schema.js'
  import { Pencil, Plus, Trash2 } from '@lucide/svelte'
  import type { ColumnDef } from '@tanstack/table-core'
  import type { AppTableFeatures } from '#lib/table-features.js'
  import { toast } from 'svelte-sonner'
  import UserMutateDrawer from './UserMutateDrawer.svelte'
  import { useUsers } from './users-state.svelte'

  const usersState = useUsers()

  const USER_SORT_IDS = new Set([
    'userId',
    'username',
    'nickName',
    'createdDate',
  ])

  let isDeleting = $state(false)

  async function fetchPage(params: {
    pageNo: number
    pageSize: number
    keyword?: string
    sort?: string
    order?: 'asc' | 'desc'
  }) {
    await usersState.loadUsers(params)
    return {
      rows: usersState.users,
      totalRowCount: usersState.totalRowCount,
    }
  }

  const columns: ColumnDef<AppTableFeatures, User>[] = [
    { accessorKey: 'username', header: '用户名' },
    { accessorKey: 'nickName', header: '昵称' },
    { accessorKey: 'email', header: '邮箱' },
    {
      accessorKey: 'active',
      header: '状态',
      cell: (info) => (info.getValue() === '1' ? '活跃' : '停用'),
    },
    {
      id: 'actions',
      header: '操作',
      cell: (info) => info.row.original,
    },
  ]

  const crudTable = useCrudTable<User>({
    columns,
    sortableIds: USER_SORT_IDS,
    fetchPage,
  })

  $effect(() => {
    void usersState.listRefetchTick
    untrack(() => {
      if (usersState.listRefetchTick > 0) {
        crudTable.reload()
      }
    })
  })

  async function handleDelete() {
    if (!usersState.currentRow) return

    isDeleting = true
    try {
      const success = await usersState.deleteUser(
        (usersState.currentRow as User).userId
      )
      if (success) {
        toast.success('用户删除成功')
        usersState.closeDialogs()
        crudTable.reload()
      }
    } finally {
      isDeleting = false
    }
  }

  function handleDeleteOpenChange(open: boolean) {
    if (!open) {
      usersState.closeDialogs()
    }
  }
</script>

<Main class="flex flex-1 flex-col gap-4 sm:gap-6">
  <div class="flex flex-wrap items-end justify-between gap-2">
    <div>
      <h2 class="text-2xl font-bold tracking-tight">用户列表</h2>
      <p class="text-muted-foreground">在此处管理您的用户和角色。</p>
    </div>
    <Button
      onclick={() => {
        usersState.setCurrentRow(null)
        usersState.setOpen('create')
      }}
    >
      <Plus class="mr-2 h-4 w-4" />
      添加用户
    </Button>
  </div>

  <CrudTableShell
    table={crudTable.table}
    isLoading={crudTable.isLoading}
    error={crudTable.error}
    columnCount={columns.length}
    searchPlaceholder="搜索用户..."
    emptyMessage="无结果。"
    bind:globalFilter={crudTable.globalFilter}
    onSearchInput={crudTable.resetPagination}
    onRefresh={() => crudTable.reload()}
  >
    {#snippet cellRenderer({ row })}
      {@const user = row as User}
      <div class="flex items-center gap-1">
        <Button
          variant="ghost"
          size="icon"
          title="编辑"
          aria-label="编辑"
          onclick={() => {
            usersState.setCurrentRow(user)
            usersState.setOpen('update')
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
            usersState.setCurrentRow(user)
            usersState.setOpen('delete')
          }}
        >
          <Trash2 class="h-4 w-4" />
        </Button>
      </div>
    {/snippet}
  </CrudTableShell>
</Main>

<UserMutateDrawer />

{#if usersState.currentRow}
  <ConfirmDialog
    destructive
    open={usersState.open === 'delete'}
    onOpenChange={handleDeleteOpenChange}
    handleConfirm={handleDelete}
    title={`删除用户 "${(usersState.currentRow as User).username}"？`}
    confirmText="删除"
    isLoading={isDeleting}
    class="max-w-md"
  >
    {#snippet desc()}
      这将软删除用户
      <strong>{(usersState.currentRow as User)?.username}</strong>。
      用户将被标记为已删除，但可以稍后恢复。
    {/snippet}
  </ConfirmDialog>
{/if}
