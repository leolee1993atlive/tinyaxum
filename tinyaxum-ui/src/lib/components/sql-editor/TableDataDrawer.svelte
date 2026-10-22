<script lang="ts">
  import { onMount } from 'svelte'
  import { sqlQueriesApi } from '$lib/api/sqlqueries.js'
  import type { QueryResultRow } from '$lib/api/types.js'
  import * as Sheet from '$lib/components/ui/sheet/index.js'
  import { LoaderCircle } from '@lucide/svelte'
  import { toast } from 'svelte-sonner'

  type QueryResultsTableType =
    typeof import('$lib/components/sql-editor/QueryResultsTable.svelte').default

  let QueryResultsTable: QueryResultsTableType | null = $state(null)

  interface Props {
    open?: boolean
    tableName?: string
    dataSourceId: number
    onOpenChange?: (open: boolean) => void
  }

  let {
    open = $bindable(false),
    tableName = '',
    dataSourceId,
    onOpenChange,
  }: Props = $props()

  let results = $state<QueryResultRow[]>([])
  let columnNames = $state<string[]>([])
  let loading = $state(false)
  let totalRows = $state(0)
  let pageIndex = $state(0)
  let pageSize = $state(20)
  let executionTime = $state(0)

  const STORAGE_KEY = 'table-preview-drawer-width'
  let drawerWidth = $state(loadWidth())
  let dragging = $state(false)

  function loadWidth(): number {
    if (typeof window === 'undefined') return 40
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved) {
      const pct = Number(saved)
      if (pct >= 20 && pct <= 80) return pct
    }
    return 40
  }

  function saveWidth() {
    localStorage.setItem(STORAGE_KEY, String(drawerWidth))
  }

  function handleDragStart(e: MouseEvent) {
    e.preventDefault()
    dragging = true

    const startX = e.clientX
    const startPct = drawerWidth
    const vw = window.innerWidth

    function onMove(ev: MouseEvent) {
      const delta = startX - ev.clientX
      const deltaPct = (delta / vw) * 100
      drawerWidth = Math.min(80, Math.max(20, startPct + deltaPct))
    }

    function onUp() {
      dragging = false
      saveWidth()
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', onUp)
    }

    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
  }

  function handleResizeKey(e: KeyboardEvent) {
    const step = e.shiftKey ? 10 : 2
    if (e.key === 'ArrowLeft') {
      drawerWidth = Math.min(80, drawerWidth + step)
    } else if (e.key === 'ArrowRight') {
      drawerWidth = Math.max(20, drawerWidth - step)
    } else {
      return
    }
    e.preventDefault()
    saveWidth()
  }

  async function loadTableData(pi = 0, ps = 20) {
    if (!tableName || dataSourceId <= 0) return
    loading = true
    const startTime = performance.now()
    try {
      const response = await sqlQueriesApi.runAdhoc({
        sqlQueryContent: `SELECT * FROM "${tableName}"`,
        dataSourceId,
        pageNo: pi + 1,
        pageSize: ps,
      })
      results = response.data?.rows ?? []
      columnNames = response.data?.columns?.map((c) => c.name) ?? []
      totalRows = response.data?.total ?? results.length
      pageIndex = pi
      pageSize = ps
      executionTime = Math.round(performance.now() - startTime)
    } catch (e) {
      toast.error('加载表数据失败')
      results = []
      columnNames = []
      totalRows = 0
      executionTime = 0
    } finally {
      loading = false
    }
  }

  $effect(() => {
    if (open && tableName && dataSourceId > 0) {
      loadTableData(0, 20)
    }
    if (!open) {
      results = []
      columnNames = []
      totalRows = 0
    }
  })

  onMount(async () => {
    const mod =
      await import('$lib/components/sql-editor/QueryResultsTable.svelte')
    QueryResultsTable = mod.default
  })
</script>

<Sheet.Root
  bind:open
  onOpenChange={(o) => {
    onOpenChange?.(o)
  }}
>
  <Sheet.Portal>
    <Sheet.Overlay class={dragging ? 'pointer-events-none' : ''} />
    <Sheet.Content
      class="flex !h-full !w-[var(--drawer-w)] !max-w-none flex-col border-l p-0"
      style="--drawer-w: {drawerWidth}%"
    >
      <!-- 可聚焦的窗口分隔条(WAI-ARIA window splitter):支持鼠标拖动 + 方向键调整宽度。
           svelte 静态检查未识别 separator 的可聚焦交互例外,故忽略下面两条误报。 -->
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div
        role="separator"
        aria-orientation="vertical"
        aria-label="拖动调整抽屉宽度"
        aria-valuenow={Math.round(drawerWidth)}
        aria-valuemin={20}
        aria-valuemax={80}
        tabindex="0"
        class="hover:bg-primary/20 active:bg-primary/30 focus-visible:bg-primary/40 absolute top-0 bottom-0 left-0 z-10 w-1 cursor-col-resize outline-none"
        onmousedown={handleDragStart}
        onkeydown={handleResizeKey}
      ></div>

      <Sheet.Header class="border-b px-4 py-3">
        <Sheet.Title class="text-base">
          {tableName}
        </Sheet.Title>
        <Sheet.Description class="text-sm">
          {loading ? '加载中...' : `${executionTime} ms`}
        </Sheet.Description>
      </Sheet.Header>

      <div class="flex-1 overflow-auto p-4">
        {#if QueryResultsTable}
          <QueryResultsTable
            data={results}
            columnOrder={columnNames}
            {loading}
            rowCount={totalRows}
            serverTotal={totalRows}
            {executionTime}
            onPageChange={(pi: number, ps: number) => loadTableData(pi, ps)}
          />
        {:else}
          <div class="flex items-center justify-center py-12">
            <LoaderCircle class="text-muted-foreground h-6 w-6 animate-spin" />
          </div>
        {/if}
      </div>
    </Sheet.Content>
  </Sheet.Portal>
</Sheet.Root>
