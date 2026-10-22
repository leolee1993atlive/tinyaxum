<script lang="ts">
  import { SvelteMap, SvelteSet } from 'svelte/reactivity'
  import { isApiClientNotifiedError } from '$lib/api/client.js'
  import { dashboardApi } from '$lib/api/dashboard.js'
  import { datasourcesApi } from '$lib/api/datasources.js'
  import { sqlQueriesApi } from '$lib/api/sqlqueries.js'
  import type {
    DashboardConfigItem,
    DashboardConfigV1,
    DatasourceListVo,
    SqlQueryDetailVo,
    SqlQueryListVo,
  } from '$lib/api/types.js'
  import DatasourceFilter from '$lib/components/crud/DatasourceFilter.svelte'
  import { Button } from '$lib/components/ui/button/index.js'
  import { Checkbox } from '$lib/components/ui/checkbox/index.js'
  import * as Command from '$lib/components/ui/command/index.js'
  import { Label } from '$lib/components/ui/label/index.js'
  import * as Popover from '$lib/components/ui/popover/index.js'
  import * as Sheet from '$lib/components/ui/sheet/index.js'
  import * as Tabs from '$lib/components/ui/tabs/index.js'
  import { cn } from '$lib/utils.js'
  import { DragDropProvider } from '@dnd-kit-svelte/svelte'
  import { useSortable } from '@dnd-kit-svelte/svelte/sortable'
  import { RestrictToVerticalAxis } from '@dnd-kit/abstract/modifiers'
  import { move } from '@dnd-kit/helpers'
  import { ChevronDown, GripVertical, Settings2, Trash2 } from '@lucide/svelte'
  import SearchIcon from '@lucide/svelte/icons/search'
  import { Command as CommandPrimitive } from 'bits-ui'
  import { toast } from 'svelte-sonner'
  import DefaultsPopover from './DefaultsPopover.svelte'

  interface Props {
    open?: boolean
    /** 是否显示内置触发按钮（默认 true，用于顶部 Header） */
    showTrigger?: boolean
    onConfigSaved?: () => void | Promise<void>
  }

  let {
    open = $bindable(false),
    showTrigger = true,
    onConfigSaved,
  }: Props = $props()

  function defaultConfig(): DashboardConfigV1 {
    return {
      version: 1,
      tabs: {
        overview: { items: [] },
        analytics: { items: [] },
        charts: { items: [] },
      },
    }
  }

  let queries = $state<SqlQueryListVo[]>([])
  let config = $state<DashboardConfigV1>(defaultConfig())
  let loading = $state(false)
  let saving = $state(false)
  let error = $state<string | null>(null)

  let datasources = $state<DatasourceListVo[]>([])

  let popoverOpen = $state<{
    overview: boolean
    analytics: boolean
    charts: boolean
  }>({ overview: false, analytics: false, charts: false })

  let dsFilter = $state<{
    overview: number | null
    analytics: number | null
    charts: number | null
  }>({ overview: null, analytics: null, charts: null })

  const selected = {
    overview: new SvelteSet<number>(),
    analytics: new SvelteSet<number>(),
    charts: new SvelteSet<number>(),
  }

  const queryDetailById = new SvelteMap<number, SqlQueryDetailVo>()

  const dsNameById = $derived(
    new Map(datasources.map((d) => [d.dataSourceId, d.name]))
  )

  $effect(() => {
    if (open) void loadAll()
  })

  const queryById = $derived(
    new Map(queries.map((q) => [q.sqlQueryId, q] as const))
  )

  function itemsForTab(
    tab: 'overview' | 'analytics' | 'charts'
  ): DashboardConfigItem[] {
    if (tab === 'overview') return config.tabs.overview.items
    if (tab === 'analytics') return config.tabs.analytics.items
    return config.tabs.charts.items
  }

  function setItemsForTab(
    tab: 'overview' | 'analytics' | 'charts',
    items: DashboardConfigItem[]
  ) {
    if (tab === 'overview') {
      config = { ...config, tabs: { ...config.tabs, overview: { items } } }
    } else if (tab === 'analytics') {
      config = { ...config, tabs: { ...config.tabs, analytics: { items } } }
    } else {
      config = { ...config, tabs: { ...config.tabs, charts: { items } } }
    }
  }

  function removeItem(
    tab: 'overview' | 'analytics' | 'charts',
    sqlQueryId: number
  ) {
    setItemsForTab(
      tab,
      itemsForTab(tab).filter((x) => x.sqlQueryId !== sqlQueryId)
    )
  }

  function addItem(
    tab: 'overview' | 'analytics' | 'charts',
    sqlQueryId: number
  ) {
    if (!Number.isFinite(sqlQueryId) || sqlQueryId <= 0) return
    const exists = itemsForTab(tab).some((x) => x.sqlQueryId === sqlQueryId)
    if (exists) return
    setItemsForTab(tab, [...itemsForTab(tab), { sqlQueryId }])
  }

  function toggleSelect(
    tab: 'overview' | 'analytics' | 'charts',
    sqlQueryId: number
  ) {
    if (selected[tab].has(sqlQueryId)) {
      selected[tab].delete(sqlQueryId)
    } else {
      selected[tab].add(sqlQueryId)
    }
  }

  function clearSelected(tab: 'overview' | 'analytics' | 'charts') {
    selected[tab].clear()
  }

  function commitAdd(tab: 'overview' | 'analytics' | 'charts') {
    for (const id of selected[tab]) {
      addItem(tab, id)
    }
    selected[tab].clear()
    popoverOpen[tab] = false
  }

  $effect(() => {
    if (!popoverOpen.overview) {
      selected.overview.clear()
      dsFilter.overview = null
    }
  })

  $effect(() => {
    if (!popoverOpen.analytics) {
      selected.analytics.clear()
      dsFilter.analytics = null
    }
  })

  $effect(() => {
    if (!popoverOpen.charts) {
      selected.charts.clear()
      dsFilter.charts = null
    }
  })

  function reorderItemsByDndEvent(
    tab: 'overview' | 'analytics' | 'charts',
    e: unknown
  ) {
    const items = [...itemsForTab(tab)]
    if (items.length <= 1) return

    const ids = items.map((c) => String(c.sqlQueryId))
    const nextIds = move(ids, e as Parameters<typeof move>[1])
    if (nextIds === ids) return

    const byId = new Map(items.map((c) => [String(c.sqlQueryId), c] as const))
    const next = nextIds
      .map((id) => byId.get(id))
      .filter((v): v is DashboardConfigItem => v != null)
    if (next.length === items.length) {
      setItemsForTab(tab, next)
    }
  }

  const addableOptions = $derived.by(() => {
    const out: Record<'overview' | 'analytics' | 'charts', SqlQueryListVo[]> = {
      overview: [],
      analytics: [],
      charts: [],
    }

    for (const tab of ['overview', 'analytics', 'charts'] as const) {
      const picked = new Set(itemsForTab(tab).map((x) => x.sqlQueryId))
      out[tab] = queries.filter((q) => {
        if (picked.has(q.sqlQueryId)) return false
        if (dsFilter[tab] != null && q.dataSourceId !== dsFilter[tab])
          return false
        if (tab === 'overview') return Number(q.isCollection) === 0
        if (tab === 'charts') return q.vizKind === 'chart'
        return (
          Number(q.isCollection) === 1 &&
          (q.vizKind === 'table' || q.vizKind == null)
        )
      })
    }
    return out
  })

  async function loadAll() {
    loading = true
    error = null
    try {
      const qres = await sqlQueriesApi.list()
      queries = Array.isArray(qres.data) ? qres.data : []
    } catch (e) {
      queries = []
      error = isApiClientNotifiedError(e)
        ? null
        : e instanceof Error
          ? e.message
          : '加载 SQL 查询列表失败'
    }
    try {
      const dres = await datasourcesApi.list()
      datasources = Array.isArray(dres.data) ? dres.data : []
    } catch {
      datasources = []
    }
    try {
      const cres = await dashboardApi.getConfig()
      config = cres.data?.config ?? defaultConfig()
    } catch {
      config = defaultConfig()
    } finally {
      loading = false
    }
    try {
      const ids = Array.from(
        new Set([
          ...config.tabs.overview.items.map((x) => x.sqlQueryId),
          ...config.tabs.analytics.items.map((x) => x.sqlQueryId),
          ...config.tabs.charts.items.map((x) => x.sqlQueryId),
        ])
      ).filter((id) => !queryDetailById.has(id))
      const limit = 4
      let i = 0
      await Promise.all(
        Array.from({ length: Math.min(limit, ids.length) }, async () => {
          while (i < ids.length) {
            const idx = i++
            try {
              const r = await sqlQueriesApi.getById(ids[idx])
              if (r.data) queryDetailById.set(r.data.sqlQueryId, r.data)
            } catch {
              // single failure silent
            }
          }
        })
      )
    } catch {
      // overall failure non-blocking
    }
  }

  async function handleSave() {
    if (saving) return
    saving = true
    error = null
    try {
      await dashboardApi.saveConfig({ config })
      toast.success('看板配置已保存')
      await onConfigSaved?.()
      open = false
    } catch (e) {
      error = isApiClientNotifiedError(e)
        ? null
        : e instanceof Error
          ? e.message
          : '保存失败'
    } finally {
      saving = false
    }
  }
</script>

{#if showTrigger}
  <Button
    data-slot="dashboard-config-trigger"
    variant="ghost"
    size="icon"
    type="button"
    aria-label="看板配置"
    onclick={() => (open = true)}
  >
    <Settings2 class="size-4" />
  </Button>
{/if}

<Sheet.Root bind:open>
  <Sheet.Content
    class="flex w-full flex-col px-4 pt-10 pb-4 sm:max-w-lg sm:px-6 sm:pb-6"
  >
    <Sheet.Header class="space-y-1.5 pr-8 text-start">
      <Sheet.Title class="flex items-center gap-2">
        <Settings2 class="size-5 shrink-0" />
        看板配置
      </Sheet.Title>
      <Sheet.Description>
        配置概览/分析/图表三个 Tab 显示哪些 SQL 查询以及排列顺序。
      </Sheet.Description>
    </Sheet.Header>

    <div class="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto py-1">
      {#if loading}
        <p class="text-muted-foreground text-sm">加载中…</p>
      {:else if error}
        <p class="text-destructive text-sm">{error}</p>
      {/if}

      <Tabs.Root value="overview" class="space-y-3">
        <Tabs.List>
          <Tabs.Trigger value="overview">概览</Tabs.Trigger>
          <Tabs.Trigger value="analytics">分析</Tabs.Trigger>
          <Tabs.Trigger value="charts">图表</Tabs.Trigger>
        </Tabs.List>

        {#each ['overview', 'analytics', 'charts'] as tab (tab)}
          <Tabs.Content value={tab} class="space-y-3">
            <div class="space-y-2">
              <Label class="text-sm font-medium">
                {tab === 'overview'
                  ? '非集合查询（卡片）'
                  : tab === 'analytics'
                    ? '集合查询（表格）'
                    : '集合查询（图表）'}
              </Label>
              <p class="text-muted-foreground text-xs">
                拖动排序；删除移除。更多可视化选项暂不在这里配置。
              </p>

              <DragDropProvider
                modifiers={[
                  // @ts-expect-error — RestrictToVerticalAxis modifier type is incompatible with DragDropProvider generics (dnd-kit type limitation)
                  RestrictToVerticalAxis,
                ]}
                onDragEnd={(e) =>
                  reorderItemsByDndEvent(
                    tab as 'overview' | 'analytics' | 'charts',
                    e
                  )}
              >
                <ul class="space-y-1.5">
                  {#each itemsForTab(tab as 'overview' | 'analytics' | 'charts') as item, i (item.sqlQueryId)}
                    {@const q = queryById.get(item.sqlQueryId)}
                    {@const { ref, isDragging, handleRef } = useSortable({
                      id: String(item.sqlQueryId),
                      index: () => i,
                    })}
                    <li
                      data-dragging={isDragging.current}
                      class="bg-muted/40 flex items-center gap-2 rounded-md border px-2 py-2 transition-[transform,opacity,box-shadow] duration-200 data-[dragging=true]:z-10 data-[dragging=true]:opacity-80 data-[dragging=true]:shadow-md"
                      role="listitem"
                      {@attach ref}
                    >
                      <button
                        type="button"
                        class="text-muted-foreground cursor-grab touch-none rounded p-0.5 active:cursor-grabbing"
                        aria-label="拖动排序"
                        {@attach handleRef}
                      >
                        <GripVertical class="size-4" />
                      </button>

                      <div class="min-w-0 flex-1">
                        <div class="truncate text-sm font-medium">
                          {q?.sqlQueryName ?? `SQL#${item.sqlQueryId}`}
                        </div>
                        {#if q?.sqlQueryDesc}
                          <div class="text-muted-foreground truncate text-xs">
                            {q.sqlQueryDesc}
                          </div>
                        {/if}
                      </div>

                      <DefaultsPopover
                        paramFields={queryDetailById.get(item.sqlQueryId)
                          ?.paramFields ?? []}
                        globalParamNames={['startDate', 'endDate']}
                        defaults={item.defaults ?? {}}
                        onChange={(next) => {
                          const items = itemsForTab(
                            tab as
                              | 'overview'
                              | 'analytics'
                              | 'charts'
                              | 'charts'
                          )
                          const idx = items.findIndex(
                            (x) => x.sqlQueryId === item.sqlQueryId
                          )
                          if (idx < 0) return
                          const nextItems = [...items]
                          nextItems[idx] = {
                            ...nextItems[idx],
                            defaults:
                              Object.keys(next).length > 0 ? next : undefined,
                          }
                          setItemsForTab(
                            tab as
                              | 'overview'
                              | 'analytics'
                              | 'charts'
                              | 'charts',
                            nextItems
                          )
                        }}
                      />

                      <Button
                        variant="ghost"
                        size="icon"
                        class={cn('size-8 shrink-0')}
                        type="button"
                        aria-label="移除"
                        title="移除"
                        onclick={() =>
                          removeItem(
                            tab as
                              | 'overview'
                              | 'analytics'
                              | 'charts'
                              | 'charts',
                            item.sqlQueryId
                          )}
                      >
                        <Trash2 class="size-4" />
                      </Button>
                    </li>
                  {/each}
                </ul>
              </DragDropProvider>
            </div>

            <div class="space-y-2">
              <Label class="text-sm font-medium">添加查询</Label>
              <Popover.Root
                bind:open={
                  popoverOpen[tab as 'overview' | 'analytics' | 'charts']
                }
              >
                <Popover.Trigger>
                  {#snippet child({ props })}
                    <Button
                      {...props}
                      variant="outline"
                      class="w-full justify-between"
                    >
                      <span class="text-muted-foreground"
                        >选择要添加的 SQL 查询…</span
                      >
                      <ChevronDown class="size-4 opacity-50" />
                    </Button>
                  {/snippet}
                </Popover.Trigger>
                <Popover.Content
                  class="w-(--bits-popover-trigger-width) max-w-[400px] p-0"
                  align="start"
                >
                  <Command.Root>
                    <div class="flex items-center gap-2 border-b p-2">
                      <DatasourceFilter
                        class="h-9 w-32 shrink-0"
                        value={dsFilter[
                          tab as 'overview' | 'analytics' | 'charts'
                        ]}
                        {datasources}
                        placeholder="全部"
                        onChange={(id) =>
                          (dsFilter[
                            tab as 'overview' | 'analytics' | 'charts'
                          ] = id)}
                      />
                      <SearchIcon class="size-4 shrink-0 opacity-50" />
                      <CommandPrimitive.Input
                        placeholder="搜索查询名…"
                        class="placeholder:text-muted-foreground h-9 flex-1 bg-transparent text-sm outline-none"
                      />
                    </div>
                    <Command.List class="max-h-[300px]">
                      <Command.Empty>未找到匹配的查询</Command.Empty>
                      {#each addableOptions[tab as 'overview' | 'analytics' | 'charts'] as q (q.sqlQueryId)}
                        <Command.Item
                          value={`${q.sqlQueryName} ${dsNameById.get(q.dataSourceId) ?? ''}`}
                          onSelect={() =>
                            toggleSelect(
                              tab as 'overview' | 'analytics' | 'charts',
                              q.sqlQueryId
                            )}
                        >
                          <Checkbox
                            checked={selected[
                              tab as 'overview' | 'analytics' | 'charts'
                            ].has(q.sqlQueryId)}
                          />
                          <span class="flex-1 truncate">{q.sqlQueryName}</span>
                          <span class="text-muted-foreground shrink-0 text-xs">
                            · {dsNameById.get(q.dataSourceId) ??
                              `DS#${q.dataSourceId}`}
                          </span>
                        </Command.Item>
                      {/each}
                    </Command.List>
                    <div class="flex items-center justify-between border-t p-2">
                      <Button
                        variant="ghost"
                        size="sm"
                        disabled={selected[
                          tab as 'overview' | 'analytics' | 'charts'
                        ].size === 0}
                        onclick={() =>
                          clearSelected(
                            tab as 'overview' | 'analytics' | 'charts'
                          )}
                      >
                        清除
                      </Button>
                      <div class="flex gap-2">
                        <Button
                          variant="outline"
                          size="sm"
                          onclick={() =>
                            (popoverOpen[
                              tab as 'overview' | 'analytics' | 'charts'
                            ] = false)}
                        >
                          关闭
                        </Button>
                        <Button
                          size="sm"
                          disabled={selected[
                            tab as 'overview' | 'analytics' | 'charts'
                          ].size === 0}
                          onclick={() =>
                            commitAdd(
                              tab as 'overview' | 'analytics' | 'charts'
                            )}
                        >
                          添加 {selected[
                            tab as 'overview' | 'analytics' | 'charts'
                          ].size} 项
                        </Button>
                      </div>
                    </div>
                  </Command.Root>
                </Popover.Content>
              </Popover.Root>
            </div>
          </Tabs.Content>
        {/each}
      </Tabs.Root>
    </div>

    <Sheet.Footer
      class="border-border -mx-4 flex flex-row flex-wrap items-center justify-end gap-2 border-t px-4 pt-4 sm:-mx-6 sm:px-6"
    >
      <Button
        type="button"
        variant="outline"
        disabled={saving}
        onclick={() => (open = false)}
      >
        关闭
      </Button>
      <Button
        type="button"
        variant="default"
        disabled={saving || loading}
        onclick={handleSave}
      >
        {saving ? '保存中…' : '保存'}
      </Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>
