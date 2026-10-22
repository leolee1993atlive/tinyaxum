<script lang="ts">
  import { onMount } from 'svelte'
  import { SvelteMap, SvelteSet } from 'svelte/reactivity'
  import { isApiClientNotifiedError } from '$lib/api/client.js'
  import { dashboardApi } from '$lib/api/dashboard.js'
  import { sqlQueriesApi } from '$lib/api/sqlqueries.js'
  import type {
    DashboardConfigItem,
    DashboardConfigV1,
    SqlQueryDetailVo,
    SqlQueryListVo,
  } from '$lib/api/types.js'
  import ProfileDropdown from '$lib/components/ProfileDropdown.svelte'
  import Search from '$lib/components/Search.svelte'
  import Header from '$lib/components/layout/Header.svelte'
  import Main from '$lib/components/layout/Main.svelte'
  import QueryResultsTable from '$lib/components/sql-editor/QueryResultsTable.svelte'
  import { Button } from '$lib/components/ui/button/index.js'
  import * as Card from '$lib/components/ui/card/index.js'
  import { Label } from '$lib/components/ui/label/index.js'
  import * as Tabs from '$lib/components/ui/tabs/index.js'
  import {
    groupDashboardItems,
    moduleItemCount,
    type GroupedItems,
    type QueryGroupMeta,
  } from '$lib/dashboard-grouping.js'
  import { cn } from '$lib/utils.js'
  import {
    Pencil,
    RefreshCw,
    Settings2,
    TrendingDown,
    TrendingUp,
  } from '@lucide/svelte'
  import ConfigDrawer from './components/ConfigDrawer.svelte'
  import DashboardChartCard from './components/DashboardChartCard.svelte'
  import DashboardGroup from './components/DashboardGroup.svelte'

  type TabKey = 'overview' | 'analytics' | 'charts'

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

  function currentMonthIso(): string {
    const d = new Date()
    const y = d.getFullYear()
    const m = String(d.getMonth() + 1).padStart(2, '0')
    return `${y}-${m}`
  }

  type RunState = {
    loading: boolean
    error: string | null
    rows: Record<string, unknown>[]
    columns: { name: string; physicalType: string }[]
    total?: number
    pageIndex?: number
    pageSize?: number
  }

  let configDrawerOpen = $state(false)
  let config = $state<DashboardConfigV1>(defaultConfig())
  let loadingConfig = $state(false)
  let refreshing = $state(false)
  let activeTab = $state<TabKey>('overview')

  let selectedMonth = $state(currentMonthIso())
  let appliedMonth = $state(currentMonthIso())
  const monthDirty = $derived(selectedMonth !== appliedMonth)

  let queriesLite = $state<SqlQueryListVo[]>([])
  const queryDetailById = new SvelteMap<number, SqlQueryDetailVo>()
  const runById = new SvelteMap<number, RunState>()

  const expanded = new SvelteSet<string>()
  const loadedGroups = new SvelteSet<string>()
  const groupLoadPromises = new SvelteMap<string, Promise<void>>()
  const metaLoadPromises = new SvelteMap<number, Promise<void>>()

  const adhocBySqlId = new SvelteMap<number, Record<string, string>>()
  const GLOBAL_PARAM_NAMES = ['startDate', 'endDate'] as const

  async function withLimit<T>(
    items: T[],
    limit: number,
    run: (x: T) => Promise<void>
  ): Promise<void> {
    let i = 0
    await Promise.all(
      Array.from({ length: Math.min(limit, items.length) }, async () => {
        while (i < items.length) {
          const idx = i++
          await run(items[idx])
        }
      })
    )
  }

  const metaById = $derived(
    new Map<number, QueryGroupMeta>(
      queriesLite.map((q) => [
        q.sqlQueryId,
        { module: q.module, category: q.category },
      ])
    )
  )
  const groupedOverview = $derived(
    groupDashboardItems(config.tabs.overview.items, metaById)
  )
  const groupedAnalytics = $derived(
    groupDashboardItems(config.tabs.analytics.items, metaById)
  )
  const groupedCharts = $derived(
    groupDashboardItems(config.tabs.charts.items, metaById)
  )
  function groupedFor(tab: TabKey): GroupedItems {
    return tab === 'overview'
      ? groupedOverview
      : tab === 'analytics'
        ? groupedAnalytics
        : groupedCharts
  }

  function noModuleKey(tab: TabKey): string {
    return `${tab}:nomodule`
  }
  function moduleKey(tab: TabKey, m: string): string {
    return `${tab}:mod:${m}`
  }
  function categoryKey(tab: TabKey, m: string, c: string): string {
    return `${tab}:cat:${m}::${c}`
  }

  function setRunState(id: number, next: RunState) {
    runById.set(id, next)
  }
  function getRunState(id: number): RunState {
    return (
      runById.get(id) ?? { loading: false, error: null, rows: [], columns: [] }
    )
  }

  function findItem(id: number): DashboardConfigItem | undefined {
    return (
      config.tabs.overview.items.find((x) => x.sqlQueryId === id) ??
      config.tabs.analytics.items.find((x) => x.sqlQueryId === id) ??
      config.tabs.charts.items.find((x) => x.sqlQueryId === id)
    )
  }

  function nonDateParams(id: number): Record<string, string> {
    const item = findItem(id)
    const out: Record<string, string> = {}
    Object.assign(out, item?.defaults ?? {})
    Object.assign(out, adhocBySqlId.get(id) ?? {})
    for (const g of GLOBAL_PARAM_NAMES) delete out[g]
    return out
  }

  // 服务端存储的配置可能缺 tab 键（如旧数据被清空成 {"tabs":{}}），
  // 逐键归一化，避免 config.tabs.xxx.items 渲染时 undefined 崩掉整页
  function normalizeConfig(raw: unknown): DashboardConfigV1 {
    const d = defaultConfig()
    const tabs = (raw as DashboardConfigV1 | null | undefined)?.tabs
    if (!tabs) return d
    for (const key of ['overview', 'analytics', 'charts'] as const) {
      const items = tabs[key]?.items
      if (Array.isArray(items)) d.tabs[key] = { items }
    }
    return d
  }

  async function loadConfig() {
    loadingConfig = true
    try {
      const res = await dashboardApi.getConfig()
      config = normalizeConfig(res.data?.config)
    } catch {
      config = defaultConfig()
    } finally {
      loadingConfig = false
    }
  }

  async function loadQueriesLite() {
    try {
      const r = await sqlQueriesApi.list()
      queriesLite = r.data ?? []
    } catch {
      queriesLite = []
    }
  }

  async function loadMetaFor(ids: number[]) {
    const missing = ids.filter((id) => !queryDetailById.has(id))
    if (missing.length === 0) return

    const toLoad: number[] = []
    const toWait: Promise<void>[] = []

    for (const id of missing) {
      const existing = metaLoadPromises.get(id)
      if (existing) {
        toWait.push(existing)
      } else {
        toLoad.push(id)
      }
    }

    if (toLoad.length > 0) {
      const promise = (async () => {
        await withLimit(toLoad, 4, async (id) => {
          try {
            const r = await sqlQueriesApi.getById(id)
            if (r.data) queryDetailById.set(r.data.sqlQueryId, r.data)
          } catch {
            // 单条失败静默
          }
        })
      })()

      toLoad.forEach((id) => metaLoadPromises.set(id, promise))
      promise.finally(() => {
        toLoad.forEach((id) => metaLoadPromises.delete(id))
      })
      toWait.push(promise)
    }

    await Promise.all(toWait)
  }

  async function refreshQuery(id: number, opts?: { noCache?: boolean }) {
    setRunState(id, { ...getRunState(id), loading: true, error: null })
    try {
      const r = await dashboardApi.runCached(
        id,
        appliedMonth,
        nonDateParams(id),
        { noCache: opts?.noCache }
      )
      const res = r.data?.result
      setRunState(id, {
        loading: false,
        error: null,
        rows: res?.rows ?? [],
        columns: res?.columns ?? [],
        total: res?.total ?? res?.rows?.length ?? 0,
      })
    } catch (e) {
      const msg = isApiClientNotifiedError(e)
        ? '运行失败'
        : e instanceof Error
          ? e.message
          : '运行失败'
      setRunState(id, { ...getRunState(id), loading: false, error: msg })
    }
  }

  async function loadGroup(key: string, ids: number[]) {
    if (loadedGroups.has(key)) return
    const existing = groupLoadPromises.get(key)
    if (existing) return existing

    const promise = (async () => {
      await loadMetaFor(ids)
      await withLimit(ids, 4, (id) => refreshQuery(id))
      loadedGroups.add(key)
    })()

    groupLoadPromises.set(key, promise)
    promise.finally(() => groupLoadPromises.delete(key))
    await promise
  }

  function toggleExpanded(key: string, open: boolean) {
    if (open) expanded.add(key)
    else expanded.delete(key)
  }

  function expandedKeysForTab(tab: TabKey): string[] {
    const grouped = groupedFor(tab)
    const keys: string[] = []
    if (expanded.has(noModuleKey(tab))) keys.push(noModuleKey(tab))
    for (const mod of grouped.modules) {
      if (expanded.has(moduleKey(tab, mod.module)))
        keys.push(moduleKey(tab, mod.module))
      for (const cat of mod.categories) {
        const k = categoryKey(tab, mod.module, cat.category)
        if (expanded.has(k)) keys.push(k)
      }
    }
    return keys
  }

  function expandedIdsForTab(tab: TabKey): number[] {
    const grouped = groupedFor(tab)
    const ids: number[] = []
    if (expanded.has(noModuleKey(tab))) {
      ids.push(...grouped.noModule.map((i) => i.sqlQueryId))
    }
    for (const mod of grouped.modules) {
      if (expanded.has(moduleKey(tab, mod.module))) {
        ids.push(...mod.directItems.map((i) => i.sqlQueryId))
      }
      for (const cat of mod.categories) {
        if (expanded.has(categoryKey(tab, mod.module, cat.category))) {
          ids.push(...cat.items.map((i) => i.sqlQueryId))
        }
      }
    }
    return Array.from(new Set(ids))
  }

  async function refreshAll() {
    if (refreshing) return
    refreshing = true
    const monthChanged = selectedMonth !== appliedMonth
    appliedMonth = selectedMonth
    try {
      if (monthChanged) {
        loadedGroups.clear()
      } else {
        await dashboardApi.invalidateMonth(appliedMonth)
      }
      for (const k of expandedKeysForTab(activeTab)) loadedGroups.add(k)
      const ids = expandedIdsForTab(activeTab)
      await withLimit(ids, 4, (id) => refreshQuery(id))
    } finally {
      refreshing = false
    }
  }

  onMount(() => {
    void loadQueriesLite()
    void loadConfig()
  })

  function queryTitle(id: number): string {
    return (
      queryDetailById.get(id)?.sqlQueryName ??
      queriesLite.find((q) => q.sqlQueryId === id)?.sqlQueryName ??
      `SQL#${id}`
    )
  }

  function scalarValueText(id: number): string {
    const detail = queryDetailById.get(id)
    const field = detail?.vizConfig?.scalarField
    if (!field) return '--'
    const row = getRunState(id).rows[0]
    if (!row) return '--'
    const v = row[field]
    return v === null || v === undefined ? '--' : String(v)
  }

  function gridClass(tab: TabKey): string {
    return tab === 'overview'
      ? 'grid gap-4 sm:grid-cols-2 lg:grid-cols-4'
      : 'space-y-4'
  }
</script>

<Header>
  <Search />
  <ProfileDropdown />
</Header>

<Main>
  <div class="mb-2 flex flex-wrap items-center justify-between gap-2 space-y-2">
    <h1 class="text-2xl font-bold tracking-tight">仪表板</h1>
    <div class="flex flex-wrap items-center gap-2">
      <div class="flex items-center gap-2">
        <Label for="dash-month" class="text-muted-foreground text-sm">
          月份
        </Label>
        <input
          id="dash-month"
          type="month"
          bind:value={selectedMonth}
          class="border-input bg-background h-9 rounded-md border px-2 text-sm"
        />
      </div>
      <Button
        type="button"
        variant={monthDirty ? 'default' : 'outline'}
        disabled={refreshing || loadingConfig}
        onclick={refreshAll}
      >
        <RefreshCw class={cn('mr-2 size-4', refreshing && 'animate-spin')} />
        刷新{monthDirty ? '（待应用）' : ''}
      </Button>
      <Button
        type="button"
        variant="outline"
        size="icon"
        aria-label="看板配置"
        title="看板配置"
        onclick={() => (configDrawerOpen = true)}
      >
        <Settings2 class="size-4" />
      </Button>
    </div>
  </div>

  <Tabs.Root
    orientation="vertical"
    value={activeTab}
    onValueChange={(v) => (activeTab = (v ?? 'overview') as TabKey)}
    class="space-y-4"
  >
    <div class="w-full overflow-x-auto pb-2">
      <Tabs.List>
        <Tabs.Trigger value="overview">概览</Tabs.Trigger>
        <Tabs.Trigger value="analytics">分析</Tabs.Trigger>
        <Tabs.Trigger value="charts">图表</Tabs.Trigger>
        <Tabs.Trigger value="notifications" disabled>通知</Tabs.Trigger>
      </Tabs.List>
    </div>

    <Tabs.Content value="overview" class="space-y-2">
      {#if loadingConfig}
        <p class="text-muted-foreground text-sm">加载配置中…</p>
      {:else if config.tabs.overview.items.length === 0}
        <p class="text-muted-foreground text-sm">
          暂无概览卡片。点击右上角"配置"选择要展示的非集合查询。
        </p>
      {:else}
        {@render groupBody('overview', groupedOverview, overviewCard)}
      {/if}
    </Tabs.Content>

    <Tabs.Content value="analytics" class="space-y-2">
      {#if loadingConfig}
        <p class="text-muted-foreground text-sm">加载配置中…</p>
      {:else if config.tabs.analytics.items.length === 0}
        <p class="text-muted-foreground text-sm">
          暂无分析表格。点击右上角"配置"选择要展示的集合查询。
        </p>
      {:else}
        {@render groupBody('analytics', groupedAnalytics, analyticsCard)}
      {/if}
    </Tabs.Content>

    <Tabs.Content value="charts" class="space-y-2">
      {#if loadingConfig}
        <p class="text-muted-foreground text-sm">加载配置中…</p>
      {:else if config.tabs.charts.items.length === 0}
        <p class="text-muted-foreground text-sm">
          暂无图表卡片。点击右上角"配置"选择 vizKind 为 chart 的集合查询。
        </p>
      {:else}
        {@render groupBody('charts', groupedCharts, chartsCard)}
      {/if}
    </Tabs.Content>
  </Tabs.Root>

  <ConfigDrawer
    bind:open={configDrawerOpen}
    showTrigger={false}
    onConfigSaved={async () => {
      await loadConfig()
      loadedGroups.clear()
    }}
  />
</Main>

{#snippet groupBody(
  tab: TabKey,
  grouped: GroupedItems,
  card: import('svelte').Snippet<[DashboardConfigItem]>
)}
  {#if grouped.noModule.length > 0}
    <DashboardGroup
      title="未归类"
      count={grouped.noModule.length}
      level={1}
      open={expanded.has(noModuleKey(tab))}
      onopenchange={(o) => toggleExpanded(noModuleKey(tab), o)}
      onexpand={() =>
        loadGroup(
          noModuleKey(tab),
          grouped.noModule.map((i) => i.sqlQueryId)
        )}
    >
      <div class={gridClass(tab)}>
        {#each grouped.noModule as item (item.sqlQueryId)}
          {@render card(item)}
        {/each}
      </div>
    </DashboardGroup>
  {/if}

  {#each grouped.modules as mod (mod.module)}
    <DashboardGroup
      title={mod.module}
      count={moduleItemCount(mod)}
      level={1}
      open={expanded.has(moduleKey(tab, mod.module))}
      onopenchange={(o) => toggleExpanded(moduleKey(tab, mod.module), o)}
      onexpand={() =>
        loadGroup(
          moduleKey(tab, mod.module),
          mod.directItems.map((i) => i.sqlQueryId)
        )}
    >
      {#if mod.directItems.length > 0}
        <div class={cn(gridClass(tab), 'mb-2')}>
          {#each mod.directItems as item (item.sqlQueryId)}
            {@render card(item)}
          {/each}
        </div>
      {/if}
      {#each mod.categories as cat (cat.category)}
        <DashboardGroup
          title={cat.category}
          count={cat.items.length}
          level={2}
          open={expanded.has(categoryKey(tab, mod.module, cat.category))}
          onopenchange={(o) =>
            toggleExpanded(categoryKey(tab, mod.module, cat.category), o)}
          onexpand={() =>
            loadGroup(
              categoryKey(tab, mod.module, cat.category),
              cat.items.map((i) => i.sqlQueryId)
            )}
        >
          <div class={gridClass(tab)}>
            {#each cat.items as item (item.sqlQueryId)}
              {@render card(item)}
            {/each}
          </div>
        </DashboardGroup>
      {/each}
    </DashboardGroup>
  {/each}
{/snippet}

{#snippet overviewCard(item: DashboardConfigItem)}
  {@const id = item.sqlQueryId}
  {@const detail = queryDetailById.get(id)}
  {#if detail && detail.isCollection !== 0}{:else}
    {@const run = getRunState(id)}
    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">{queryTitle(id)}</Card.Title>
        <div class="flex shrink-0 items-center gap-1">
          <Button
            type="button"
            variant="ghost"
            size="icon"
            class="size-8"
            aria-label="刷新"
            title="刷新"
            onclick={() => refreshQuery(id, { noCache: true })}
          >
            <RefreshCw class="size-4" />
          </Button>
          <a
            href={`/sqlquery?edit=${id}`}
            aria-label="编辑 SQL 查询"
            title="编辑 SQL 查询"
          >
            <Button type="button" variant="ghost" size="icon" class="size-8">
              <Pencil class="size-4" />
            </Button>
          </a>
        </div>
      </Card.Header>
      <Card.Content class="space-y-1">
        <div class="text-center text-4xl font-bold tabular-nums">
          {run.loading ? '—' : scalarValueText(id)}
        </div>
        <div
          class="text-muted-foreground flex items-center justify-between text-xs"
        >
          <span class="inline-flex items-center gap-1">
            <TrendingUp class="size-3" />
            同比 --
          </span>
          <span class="inline-flex items-center gap-1">
            <TrendingDown class="size-3" />
            环比 --
          </span>
        </div>
        {#if run.error}
          <p class="text-destructive text-xs">{run.error}</p>
        {/if}
      </Card.Content>
    </Card.Root>
  {/if}
{/snippet}

{#snippet analyticsCard(item: DashboardConfigItem)}
  {@const id = item.sqlQueryId}
  {@const detail = queryDetailById.get(id)}
  {#if detail && detail.isCollection !== 1}{:else}
    {@const run = getRunState(id)}
    <Card.Root>
      <Card.Header class="flex items-start justify-between gap-2">
        <div class="space-y-1">
          <Card.Title>{queryTitle(id)}</Card.Title>
          <Card.Description>
            表头与顺序来自该查询的可视化配置（vizConfig）。
          </Card.Description>
        </div>
        <div class="flex shrink-0 items-center gap-1">
          <Button
            type="button"
            variant="ghost"
            size="icon"
            class="size-8 shrink-0"
            aria-label="刷新"
            title="刷新"
            onclick={() => refreshQuery(id, { noCache: true })}
          >
            <RefreshCw class="size-4" />
          </Button>
          <a
            href={`/sqlquery?edit=${id}`}
            aria-label="编辑 SQL 查询"
            title="编辑 SQL 查询"
          >
            <Button
              type="button"
              variant="ghost"
              size="icon"
              class="size-8 shrink-0"
            >
              <Pencil class="size-4" />
            </Button>
          </a>
        </div>
      </Card.Header>
      <Card.Content>
        {#if run.error}
          <p class="text-destructive mb-2 text-sm">{run.error}</p>
        {/if}
        <QueryResultsTable
          data={run.rows}
          loading={run.loading}
          rowCount={run.total ?? run.rows.length}
          executionTime={0}
          vizTable={detail?.vizConfig?.table}
          resultFields={detail?.resultFields ?? []}
        />
      </Card.Content>
    </Card.Root>
  {/if}
{/snippet}

{#snippet chartsCard(item: DashboardConfigItem)}
  {@const id = item.sqlQueryId}
  {@const detail = queryDetailById.get(id)}
  {#if detail && (detail.isCollection !== 1 || detail.vizConfig?.kind !== 'chart')}{:else}
    {@const run = getRunState(id)}
    <DashboardChartCard
      title={queryTitle(id)}
      {detail}
      rows={run.rows}
      columns={run.columns}
      loading={run.loading}
      error={run.error}
      initialAdhocParams={{
        ...(item.defaults ?? {}),
        ...(adhocBySqlId.get(id) ?? {}),
      }}
      globalParamNames={[...GLOBAL_PARAM_NAMES]}
      sqlQueryId={id}
      onRefresh={() => refreshQuery(id, { noCache: true })}
      onApply={(adhoc) => {
        const filtered: Record<string, string> = {}
        const globals = new Set<string>(GLOBAL_PARAM_NAMES)
        for (const [k, v] of Object.entries(adhoc)) {
          if (!globals.has(k)) filtered[k] = v
        }
        adhocBySqlId.set(id, filtered)
        void refreshQuery(id)
      }}
    />
  {/if}
{/snippet}
