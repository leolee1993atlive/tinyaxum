<script lang="ts">
  import { untrack } from 'svelte'
  import type { AiProviderListVo } from '$lib/api/types'
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte'
  import { CrudTableShell } from '$lib/components/crud/index.js'
  import Main from '$lib/components/layout/Main.svelte'
  import { Button } from '$lib/components/ui/button/index.js'
  import { useCrudTable } from '$lib/composables/use-crud-table.svelte.js'
  import {
    Pencil,
    Plus,
    Trash2,
    Star,
    LoaderCircle,
    SquareActivity,
    Copy,
  } from '@lucide/svelte'
  import type { ColumnDef } from '@tanstack/table-core'
  import { toast } from 'svelte-sonner'
  import { useAiProviders } from './ai-provider-state.svelte'

  const aiProviderCtx = useAiProviders()

  let isDeleting = $state(false)
  let testingId = $state<number | null>(null)

  async function fetchPage(_params: {
    pageNo: number
    pageSize: number
    keyword?: string
    sort?: string
    order?: 'asc' | 'desc'
  }) {
    return aiProviderCtx.loadProviders()
  }

  const columns: ColumnDef<AiProviderListVo>[] = [
    { accessorKey: 'name', header: '名称' },
    { accessorKey: 'model', header: '模型' },
    { accessorKey: 'baseUrl', header: 'Base URL' },
    {
      accessorKey: 'apiKeyMasked',
      header: 'API Key',
      cell: (info) => (info.getValue() as string) || '-',
    },
    {
      id: 'actions',
      header: '操作',
      cell: (info) => info.row.original,
    },
  ]

  const crudTable = useCrudTable<AiProviderListVo>({
    columns,
    sortableIds: new Set(['name', 'model', 'baseUrl']),
    fetchPage,
  })

  $effect(() => {
    void aiProviderCtx.listRefetchTick
    untrack(() => {
      if (aiProviderCtx.listRefetchTick > 0) {
        crudTable.reload()
      }
    })
  })

  async function handleSetDefault(provider: AiProviderListVo) {
    const ok = await aiProviderCtx.setDefault(provider.aiProviderId)
    if (ok) {
      toast.success('已设为默认')
      crudTable.reload()
    } else if (aiProviderCtx.error) {
      toast.error(aiProviderCtx.error)
    }
  }

  async function handleTest(provider: AiProviderListVo) {
    testingId = provider.aiProviderId
    try {
      const result = await aiProviderCtx.testProvider(provider.aiProviderId)
      if (result?.ok) {
        toast.success(
          `连接成功 ${result.latencyMs}ms${result.modelEcho ? ` · ${result.modelEcho}` : ''}`
        )
      } else {
        toast.error(`连接失败: ${result?.error ?? '未知错误'}`)
      }
    } catch (e) {
      toast.error(`测试失败: ${e}`)
    } finally {
      testingId = null
    }
  }

  async function handleDelete() {
    if (!aiProviderCtx.currentRow) return
    isDeleting = true
    try {
      const success = await aiProviderCtx.deleteProvider(
        aiProviderCtx.currentRow.aiProviderId
      )
      if (success) {
        toast.success('删除成功')
        aiProviderCtx.closeDialogs()
        crudTable.reload()
      }
    } finally {
      isDeleting = false
    }
  }

  function handleDeleteOpenChange(open: boolean) {
    if (!open) {
      aiProviderCtx.closeDialogs()
    }
  }
</script>

<Main class="flex flex-1 flex-col gap-4 sm:gap-6">
  <div class="flex flex-wrap items-end justify-between gap-2">
    <div>
      <h2 class="text-2xl font-bold tracking-tight">AI 模型</h2>
      <p class="text-muted-foreground">管理 OpenAI 兼容的 LLM 供应商配置。</p>
    </div>
    <Button
      onclick={() => {
        aiProviderCtx.setCurrentRow(null)
        aiProviderCtx.setOpen('create')
      }}
    >
      <Plus class="mr-2 h-4 w-4" />
      新增模型
    </Button>
  </div>

  <CrudTableShell
    table={crudTable.table}
    isLoading={crudTable.isLoading}
    error={crudTable.error}
    columnCount={columns.length}
    searchPlaceholder="搜索模型..."
    emptyMessage="暂无 AI 模型，请新增。"
    bind:globalFilter={crudTable.globalFilter}
    onSearchInput={crudTable.resetPagination}
    onRefresh={() => crudTable.reload()}
  >
    {#snippet cellRenderer({ row })}
      {@const provider = row as AiProviderListVo}
      <div class="flex items-center gap-1">
        <Button
          variant="ghost"
          size="icon"
          title={provider.isDefault === 1 ? '当前默认' : '设为默认'}
          aria-label="设为默认"
          onclick={() => handleSetDefault(provider)}
        >
          <Star
            class={provider.isDefault === 1
              ? 'h-4 w-4 fill-yellow-400 text-yellow-400'
              : 'text-muted-foreground h-4 w-4'}
          />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          title="测试连接"
          aria-label="测试连接"
          onclick={() => handleTest(provider)}
          disabled={testingId === provider.aiProviderId}
        >
          {#if testingId === provider.aiProviderId}
            <LoaderCircle class="h-4 w-4 animate-spin" />
          {:else}
            <SquareActivity class="size-4" />
          {/if}
        </Button>
        <Button
          variant="ghost"
          size="icon"
          title="复制"
          aria-label="复制"
          onclick={async () => {
            const ok = await aiProviderCtx.copyProvider(provider.aiProviderId)
            if (!ok && aiProviderCtx.error) {
              toast.error(aiProviderCtx.error)
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
            aiProviderCtx.setCurrentRow(provider)
            aiProviderCtx.setOpen('update')
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
          onclick={() => aiProviderCtx.confirmDelete(provider)}
        >
          <Trash2 class="h-4 w-4" />
        </Button>
      </div>
    {/snippet}
  </CrudTableShell>
</Main>

{#if aiProviderCtx.currentRow}
  <ConfirmDialog
    destructive
    open={aiProviderCtx.open === 'delete'}
    onOpenChange={handleDeleteOpenChange}
    handleConfirm={handleDelete}
    title={`删除模型 "${aiProviderCtx.currentRow.name}"？`}
    confirmText="删除"
    isLoading={isDeleting}
    class="max-w-md"
  >
    {#snippet desc()}
      这将删除 AI 模型
      <strong>{aiProviderCtx.currentRow?.name}</strong>。
      引用此模型的会话仍可查看历史，但发送新消息会报错。
    {/snippet}
  </ConfirmDialog>
{/if}
