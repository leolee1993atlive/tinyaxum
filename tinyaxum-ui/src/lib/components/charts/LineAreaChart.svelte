<script lang="ts">
  import { onMount } from 'svelte'
  import type {
    QueryResultRow,
    SqlQueryVizChartSeries,
  } from '$lib/api/types.js'
  import { Area, Axis, Chart, Spline, Svg } from 'layerchart'

  interface Props {
    rows: QueryResultRow[]
    xField: string
    series: SqlQueryVizChartSeries[]
    xType: 'date' | 'category'
    height?: number
  }

  let { rows, xField, series, xType, height = 280 }: Props = $props()

  let mounted = $state(false)
  onMount(() => {
    mounted = true
  })

  function colorForIndex(i: number): string {
    const n = (i % 5) + 1
    return `var(--chart-${n})`
  }

  function toNumber(v: unknown): number | null {
    if (v === null || v === undefined || v === '') return null
    const n = typeof v === 'number' ? v : Number(v)
    return Number.isFinite(n) ? n : null
  }

  function toX(v: unknown): Date | string | number {
    if (xType === 'date') {
      if (v instanceof Date) return v
      const d = new Date(String(v))
      return Number.isNaN(d.getTime()) ? String(v ?? '') : d
    }
    if (typeof v === 'number') return v
    return String(v ?? '')
  }

  const data = $derived(
    rows.map((row) => {
      const out: Record<string, unknown> = { x: toX(row[xField]) }
      for (const s of series) {
        out[s.fieldName] = toNumber(row[s.fieldName])
      }
      return out
    })
  )

  const seriesData = $derived(
    series.map((s, i) => ({
      key: s.fieldName,
      label: s.label ?? s.fieldName,
      color: colorForIndex(i),
      type: s.type,
    }))
  )
</script>

{#if !mounted}
  <div
    class="text-muted-foreground flex items-center justify-center"
    style="height: {height}px"
  >
    渲染中…
  </div>
{:else if data.length === 0 || series.length === 0}
  <div
    class="text-muted-foreground flex items-center justify-center"
    style="height: {height}px"
  >
    无数据
  </div>
{:else}
  <div style="height: {height}px">
    <Chart
      {data}
      x="x"
      y={(d: Record<string, unknown>) => {
        let max = -Infinity
        for (const s of series) {
          const v = d[s.fieldName]
          if (typeof v === 'number' && v > max) max = v
        }
        return max === -Infinity ? 0 : max
      }}
      padding={{ left: 16, bottom: 24, top: 8, right: 8 }}
      tooltip={{ mode: 'bisect-x' }}
    >
      {#snippet children()}
        <Svg>
          <Axis placement="bottom" />
          <Axis placement="left" />
          {#each seriesData as s (s.key)}
            {#if s.type === 'area'}
              <Area
                y={s.key}
                fill={s.color}
                fillOpacity={0.2}
                line={{ stroke: s.color, 'stroke-width': 1.5 }}
              />
            {:else}
              <Spline y={s.key} stroke={s.color} stroke-width={2} />
            {/if}
          {/each}
        </Svg>
      {/snippet}
    </Chart>
  </div>
{/if}
