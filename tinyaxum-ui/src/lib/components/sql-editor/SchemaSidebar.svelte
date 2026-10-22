<script lang="ts">
  import { untrack } from 'svelte'
  import { isApiClientNotifiedError } from '$lib/api/client.js'
  import { datasourcesApi } from '$lib/api/datasources.js'
  import { sqlQueriesApi } from '$lib/api/sqlqueries.js'
  import type {
    DatasourceColumnSchemaVo,
    DatasourceTableSchemaVo,
  } from '$lib/api/types.js'
  import * as Accordion from '$lib/components/ui/accordion/index.js'
  import { Button } from '$lib/components/ui/button/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js'
  import { cn } from '$lib/utils.js'
  import {
    ChevronDown,
    Database,
    Eye,
    RefreshCw,
    Search,
    Table2,
  } from '@lucide/svelte'
  import { Accordion as AccordionPrimitive } from 'bits-ui'
  import { toast } from 'svelte-sonner'

  interface Props {
    dataSourceId: number
    oninsert: (text: string) => void
    onpreviewtable?: (tableName: string) => void
    onschemaload?: (tables: DatasourceTableSchemaVo[]) => void
  }

  let { dataSourceId, oninsert, onpreviewtable, onschemaload }: Props = $props()

  // 按数据源 id 缓存,切换回来不重复请求
  let schemaCache = $state<Record<number, DatasourceTableSchemaVo[]>>({})
  let loadingId = $state(0)
  let refreshing = $state(false)
  let searchQuery = $state('')
  let openItem = $state('')

  let tables = $derived(schemaCache[dataSourceId] ?? [])
  let loading = $derived(loadingId === dataSourceId && dataSourceId > 0)

  $effect(() => {
    const id = dataSourceId
    // 切换数据源时重置展开/搜索状态
    openItem = ''
    searchQuery = ''
    untrack(() => {
      if (id > 0) {
        if (id in schemaCache) {
          onschemaload?.(schemaCache[id])
        } else {
          void loadSchema(id)
        }
      }
    })
  })

  async function loadSchema(id: number) {
    loadingId = id
    try {
      const res = await sqlQueriesApi.getQuerySchema(id)
      schemaCache[id] = res.data?.tables ?? []
      onschemaload?.(schemaCache[id])
    } catch (e) {
      if (!isApiClientNotifiedError(e)) {
        toast.error('加载元数据失败')
      }
      schemaCache[id] = []
      onschemaload?.([])
    } finally {
      if (loadingId === id) loadingId = 0
    }
  }

  async function handleRefresh() {
    const id = dataSourceId
    if (id <= 0 || refreshing) return
    refreshing = true
    try {
      await datasourcesApi.refreshSchema(id)
      delete schemaCache[id]
      await loadSchema(id)
      toast.success('元数据已刷新')
    } catch (e) {
      if (!isApiClientNotifiedError(e)) {
        toast.error('刷新元数据失败')
      }
    } finally {
      refreshing = false
    }
  }

  function tableMatches(t: DatasourceTableSchemaVo, q: string): boolean {
    return (
      t.tableName.toLowerCase().includes(q) ||
      (t.commentLabel?.toLowerCase().includes(q) ?? false)
    )
  }

  function columnMatches(c: DatasourceColumnSchemaVo, q: string): boolean {
    return (
      c.columnName.toLowerCase().includes(q) ||
      (c.commentLabel?.toLowerCase().includes(q) ?? false)
    )
  }

  let filteredTables = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase()
    if (!q) return tables
    return tables.filter(
      (t) => tableMatches(t, q) || t.columns.some((c) => columnMatches(c, q))
    )
  })

  // 表名命中时展示全部列;仅列命中时只展示匹配的列
  function visibleColumns(
    t: DatasourceTableSchemaVo
  ): DatasourceColumnSchemaVo[] {
    const q = searchQuery.trim().toLowerCase()
    if (!q || tableMatches(t, q)) return t.columns
    return t.columns.filter((c) => columnMatches(c, q))
  }
</script>

<div
  data-slot="schema-sidebar"
  class="bg-background flex h-full min-h-0 flex-col border-l"
>
  <div
    class="flex shrink-0 items-center justify-between gap-2 border-b px-3 py-2.5"
  >
    <div class="flex min-w-0 items-center gap-2">
      <Database class="text-muted-foreground size-4 shrink-0" />
      <span class="truncate text-sm font-medium">元数据</span>
    </div>
    <Button
      variant="outline"
      size="sm"
      class="h-8 shrink-0 gap-1 px-2 text-xs"
      onclick={handleRefresh}
      disabled={dataSourceId <= 0 || refreshing || loading}
      title="从源库重新采集元数据"
    >
      <RefreshCw class={cn('size-3.5', refreshing && 'animate-spin')} />
      刷新
    </Button>
  </div>

  <div class="shrink-0 border-b px-3 py-2">
    <div class="relative">
      <Search
        class="text-muted-foreground absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2"
      />
      <Input
        type="text"
        placeholder="搜索表或字段..."
        bind:value={searchQuery}
        disabled={dataSourceId <= 0}
        class="h-8 pl-8 text-xs"
      />
    </div>
  </div>

  <ScrollArea class="min-h-0 flex-1">
    {#if dataSourceId <= 0}
      <div class="flex flex-col items-center justify-center gap-1.5 px-4 py-8">
        <Database class="text-muted-foreground/50 size-8" />
        <span class="text-muted-foreground text-xs">请先选择数据源</span>
      </div>
    {:else if loading}
      <div class="flex items-center justify-center py-8">
        <span class="text-muted-foreground text-xs">加载...</span>
      </div>
    {:else if filteredTables.length === 0}
      <div class="flex flex-col items-center justify-center gap-1.5 px-4 py-8">
        <Database class="text-muted-foreground/50 size-8" />
        <span class="text-muted-foreground text-xs">
          {#if searchQuery.trim()}
            没有表或字段匹配 "{searchQuery}"
          {:else}
            该数据源暂无已导出到 DuckDB 的表(请先在传输任务中同步)
          {/if}
        </span>
      </div>
    {:else}
      <Accordion.Root
        type="single"
        class="w-full px-2 py-1"
        bind:value={openItem}
      >
        {#each filteredTables as table (table.tableMetaId)}
          <Accordion.Item value={String(table.tableMetaId)} class="border-b-0">
            <AccordionPrimitive.Trigger
              class={cn(
                'group/trigger hover:bg-accent flex w-full min-w-0 items-center gap-2 rounded-md px-2 py-2 text-left transition-colors',
                'focus-visible:ring-ring/50 border border-transparent outline-none focus-visible:ring-2 focus-visible:ring-offset-2'
              )}
            >
              <Table2 class="text-muted-foreground size-3.5 shrink-0" />
              <span class="min-w-0 flex-1">
                <span
                  class="block truncate text-sm leading-tight font-medium"
                  title={table.commentLabel || table.tableName}
                >
                  {table.commentLabel || table.tableName}
                </span>
                {#if table.commentLabel && table.commentLabel !== table.tableName}
                  <span
                    class="text-muted-foreground block truncate font-mono text-[11px] leading-tight"
                  >
                    {table.tableName}
                  </span>
                {/if}
              </span>
              <ChevronDown
                class="text-muted-foreground size-4 shrink-0 transition-transform duration-200 group-data-[state=open]/trigger:rotate-180"
              />
              <button
                type="button"
                class="hover:bg-accent/50 shrink-0 rounded p-1 transition-colors"
                title={`查看 ${table.tableName} 数据`}
                onclick={(e) => {
                  e.stopPropagation()
                  onpreviewtable?.(table.tableName)
                }}
              >
                <Eye class="text-muted-foreground size-3.5" />
              </button>
            </AccordionPrimitive.Trigger>
            <Accordion.Content class="px-1 pt-0 pb-1">
              <div class="space-y-0.5">
                {#each visibleColumns(table) as column (column.columnMetaId)}
                  <button
                    type="button"
                    class="hover:bg-accent flex w-full min-w-0 items-center gap-2 rounded-md px-2 py-1.5 text-left transition-colors"
                    title={`点击插入字段名:${column.columnName}`}
                    onclick={() => oninsert(column.columnName)}
                  >
                    <span class="min-w-0 flex-1">
                      <span
                        class="block truncate font-mono text-xs leading-tight"
                      >
                        {column.columnName}
                      </span>
                      {#if column.commentLabel}
                        <span
                          class="text-muted-foreground block truncate text-[11px] leading-tight"
                        >
                          {column.commentLabel}
                        </span>
                      {/if}
                    </span>
                    <span
                      class="bg-muted text-muted-foreground shrink-0 rounded px-1.5 py-0.5 font-mono text-[10px]"
                    >
                      {column.fieldType}
                    </span>
                  </button>
                {/each}
              </div>
            </Accordion.Content>
          </Accordion.Item>
        {/each}
      </Accordion.Root>
    {/if}
  </ScrollArea>
</div>
