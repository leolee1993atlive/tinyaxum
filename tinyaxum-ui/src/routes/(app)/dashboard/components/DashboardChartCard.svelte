<script lang="ts">
  import { untrack } from 'svelte'
  import type {
    QueryResultRow,
    SqlQueryDetailVo,
    SqlResultColumnVo,
  } from '$lib/api/types.js'
  import LineAreaChart from '$lib/components/charts/LineAreaChart.svelte'
  import { Button } from '$lib/components/ui/button/index.js'
  import * as Card from '$lib/components/ui/card/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import { Skeleton } from '$lib/components/ui/skeleton/index.js'
  import { cn } from '$lib/utils.js'
  import { Pencil, RefreshCw, Settings2 } from '@lucide/svelte'

  interface Props {
    title: string
    detail?: SqlQueryDetailVo
    rows: QueryResultRow[]
    columns: SqlResultColumnVo[]
    loading: boolean
    error: string | null
    initialAdhocParams: Record<string, string>
    globalParamNames: string[]
    sqlQueryId: number
    onRefresh?: () => void
    onApply: (adhoc: Record<string, string>) => void
  }

  let {
    title,
    detail,
    rows,
    columns,
    loading,
    error,
    initialAdhocParams,
    globalParamNames,
    sqlQueryId,
    onRefresh,
    onApply,
  }: Props = $props()

  let paramsOpen = $state(false)
  let adhoc = $state<Record<string, string>>(
    untrack(() => ({ ...initialAdhocParams }))
  )

  const nonGlobalParams = $derived(
    (detail?.paramFields ?? []).filter(
      (p) => !globalParamNames.includes(p.fieldName)
    )
  )

  const missingRequired = $derived(
    nonGlobalParams
      .filter((p) => p.required)
      .filter((p) => !adhoc[p.fieldName] || adhoc[p.fieldName] === '')
      .map((p) => p.fieldName)
  )

  const xField = $derived(detail?.vizConfig?.chart?.xField ?? '')
  const series = $derived(detail?.vizConfig?.chart?.series ?? [])

  const xType = $derived.by<'date' | 'category'>(() => {
    if (!xField) return 'category'
    const col = columns.find((c) => c.name === xField)
    const t = col?.physicalType?.toUpperCase() ?? ''
    return t.includes('DATE') || t.includes('TIME') ? 'date' : 'category'
  })

  function applyForm() {
    onApply({ ...adhoc })
  }
</script>

<Card.Root>
  <Card.Header class="flex items-start justify-between gap-2">
    <div class="space-y-1">
      <Card.Title>{title}</Card.Title>
      {#if detail?.sqlQueryDesc}
        <Card.Description>{detail.sqlQueryDesc}</Card.Description>
      {/if}
    </div>
    <div class="flex shrink-0 items-center gap-1">
      {#if onRefresh}
        <Button
          type="button"
          variant="ghost"
          size="icon"
          class="size-8"
          aria-label="刷新"
          title="刷新"
          onclick={() => onRefresh?.()}
        >
          <RefreshCw class="size-4" />
        </Button>
      {/if}
      {#if nonGlobalParams.length > 0}
        <Button
          type="button"
          variant="ghost"
          size="icon"
          class="size-8"
          aria-label="参数"
          title="参数"
          onclick={() => (paramsOpen = !paramsOpen)}
        >
          <Settings2 class="size-4" />
        </Button>
      {/if}
      <a
        href={`/sqlquery?edit=${sqlQueryId}`}
        aria-label="编辑 SQL 查询"
        title="编辑 SQL 查询"
      >
        <Button type="button" variant="ghost" size="icon" class="size-8">
          <Pencil class="size-4" />
        </Button>
      </a>
    </div>
  </Card.Header>

  {#if paramsOpen && nonGlobalParams.length > 0}
    <Card.Content class="space-y-2">
      <div class="flex flex-wrap items-end gap-2">
        {#each nonGlobalParams as p (p.fieldName)}
          <div class="flex flex-col gap-1">
            <label
              class={cn(
                'text-muted-foreground text-xs',
                missingRequired.includes(p.fieldName) && 'text-destructive'
              )}
              for={`p-${sqlQueryId}-${p.fieldName}`}
            >
              {p.commentLabel || p.fieldName}{p.required ? ' *' : ''}
            </label>
            <Input
              id={`p-${sqlQueryId}-${p.fieldName}`}
              value={adhoc[p.fieldName] ?? ''}
              oninput={(e) =>
                (adhoc = {
                  ...adhoc,
                  [p.fieldName]: e.currentTarget.value,
                })}
              class={cn(
                'h-8 w-40',
                missingRequired.includes(p.fieldName) && 'border-destructive'
              )}
            />
          </div>
        {/each}
        <Button
          type="button"
          variant="default"
          size="sm"
          onclick={applyForm}
          disabled={missingRequired.length > 0}
        >
          应用
        </Button>
      </div>
    </Card.Content>
  {/if}

  <Card.Content>
    {#if missingRequired.length > 0}
      <div
        class="text-destructive flex items-center justify-center text-sm"
        style="height: 280px"
      >
        参数缺失: {missingRequired.join(', ')}
      </div>
    {:else if error}
      <div
        class="text-destructive flex items-center justify-center text-sm"
        style="height: 280px"
      >
        {error}
      </div>
    {:else if loading}
      <Skeleton class="h-[280px] w-full" />
    {:else if !xField || series.length === 0}
      <div
        class="text-muted-foreground flex items-center justify-center text-sm"
        style="height: 280px"
      >
        未配置图表(请到 SQL 编辑器配置 X 轴与系列)
      </div>
    {:else}
      <LineAreaChart {rows} {xField} {series} {xType} height={280} />
    {/if}
  </Card.Content>
</Card.Root>
