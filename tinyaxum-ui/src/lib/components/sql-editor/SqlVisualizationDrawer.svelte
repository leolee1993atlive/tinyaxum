<script lang="ts">
  import type {
    SqlQueryCollectionFlag,
    SqlQueryResultFieldVo,
    SqlQueryVizChart,
    SqlQueryVizChartSeries,
    SqlQueryVizConfig,
    SqlQueryVizTableColumn,
  } from '$lib/api/types.js'
  import { Button } from '$lib/components/ui/button/index.js'
  import { Checkbox } from '$lib/components/ui/checkbox/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import { Label } from '$lib/components/ui/label/index.js'
  import * as Popover from '$lib/components/ui/popover/index.js'
  import * as Select from '$lib/components/ui/select/index.js'
  import * as Sheet from '$lib/components/ui/sheet/index.js'
  import {
    isNumericFieldType,
    normalizeVizConfig,
    type VizFieldMeta,
  } from '$lib/sql-query/viz-config.js'
  import { cn } from '$lib/utils.js'
  import { DragDropProvider } from '@dnd-kit-svelte/svelte'
  import { useSortable } from '@dnd-kit-svelte/svelte/sortable'
  import { RestrictToVerticalAxis } from '@dnd-kit/abstract/modifiers'
  import { move } from '@dnd-kit/helpers'
  import {
    Eye,
    EyeOff,
    GripVertical,
    MoreHorizontal,
    Settings2,
    Trash2,
  } from '@lucide/svelte'

  interface Props {
    open?: boolean
    isCollection: SqlQueryCollectionFlag
    resultFields?: SqlQueryResultFieldVo[]
    previewFieldNames?: string[]
    vizConfig: SqlQueryVizConfig
    /** 传入时显示「保存」按钮；抛错则保持抽屉打开 */
    onVizConfigPersist?: () => void | Promise<void>
  }

  let {
    open = $bindable(false),
    isCollection,
    resultFields = [],
    previewFieldNames = [],
    vizConfig = $bindable(),
    onVizConfigPersist,
  }: Props = $props()

  let saving = $state(false)

  let lastTable = $state<SqlQueryVizConfig['table']>(undefined)
  let lastChart = $state<SqlQueryVizChart | undefined>(undefined)

  $effect(() => {
    if (vizConfig.kind === 'chart' && vizConfig.chart) {
      lastChart = vizConfig.chart
    }
    if (
      (vizConfig.kind === 'table' || vizConfig.kind === undefined) &&
      vizConfig.table
    ) {
      lastTable = vizConfig.table
    }
  })

  function switchKind(kind: 'table' | 'chart') {
    if (kind === 'chart') {
      const next: SqlQueryVizConfig = {
        version: 1,
        kind: 'chart',
        chart: lastChart,
      }
      vizConfig = normalizeVizConfig(next, isCollection, effectiveFields)
    } else {
      const next: SqlQueryVizConfig = {
        version: 1,
        kind: 'table',
        table: lastTable,
      }
      vizConfig = normalizeVizConfig(next, isCollection, effectiveFields)
    }
  }

  function updateChartXField(name: string) {
    const cur = vizConfig.chart
    if (!cur) return
    vizConfig = {
      ...vizConfig,
      kind: 'chart',
      chart: {
        ...cur,
        xField: name,
        series: cur.series.filter((s) => s.fieldName !== name),
      },
    }
  }

  function updateChartSeries(
    i: number,
    patch: Partial<SqlQueryVizChartSeries>
  ) {
    const cur = vizConfig.chart
    if (!cur) return
    const next = [...cur.series]
    if (!next[i]) return
    next[i] = { ...next[i], ...patch }
    vizConfig = {
      ...vizConfig,
      kind: 'chart',
      chart: { ...cur, series: next },
    }
  }

  function removeChartSeries(i: number) {
    const cur = vizConfig.chart
    if (!cur) return
    const next = cur.series.filter((_, idx) => idx !== i)
    vizConfig = {
      ...vizConfig,
      kind: 'chart',
      chart: { ...cur, series: next },
    }
  }

  function addChartSeries() {
    const cur = vizConfig.chart
    if (!cur) return
    const used = new Set([cur.xField, ...cur.series.map((s) => s.fieldName)])
    const candidate = effectiveFields.find((f) => !used.has(f.fieldName))
    if (!candidate) return
    vizConfig = {
      ...vizConfig,
      kind: 'chart',
      chart: {
        ...cur,
        series: [
          ...cur.series,
          {
            fieldName: candidate.fieldName,
            type: 'line',
            label: candidate.commentLabel || candidate.fieldName,
          },
        ],
      },
    }
  }

  function handleClose() {
    if (saving) return
    open = false
  }

  async function handleSave() {
    if (!onVizConfigPersist || saving) return
    saving = true
    try {
      await onVizConfigPersist()
    } catch {
      return
    } finally {
      saving = false
    }
    open = false
  }

  const effectiveFields = $derived<VizFieldMeta[]>(
    resultFields.length > 0
      ? resultFields.map((f) => ({
          fieldName: f.fieldName,
          commentLabel: f.commentLabel,
          fieldType: f.fieldType,
        }))
      : previewFieldNames.map((n) => ({ fieldName: n, fieldType: 'TEXT' }))
  )

  function ensureTable(): SqlQueryVizConfig {
    const n = normalizeVizConfig(vizConfig, 1, effectiveFields)
    vizConfig = n
    return n
  }

  function tableColumns(): SqlQueryVizTableColumn[] {
    return vizConfig.table?.columns ?? []
  }

  function setTableColumns(cols: SqlQueryVizTableColumn[]) {
    const base = ensureTable()
    vizConfig = {
      ...base,
      table: {
        showRowIndex: base.table?.showRowIndex === true,
        columns: cols,
      },
    }
  }

  function toggleShowRowIndex(v: boolean) {
    const t = ensureTable()
    vizConfig = {
      ...t,
      table: {
        ...t.table!,
        showRowIndex: v,
      },
    }
  }

  function toggleColumnVisible(i: number) {
    const cols = [...tableColumns()]
    if (!cols[i]) return
    cols[i] = { ...cols[i], visible: !cols[i].visible }
    setTableColumns(cols)
  }

  function updateColumn(i: number, patch: Partial<SqlQueryVizTableColumn>) {
    const cols = [...tableColumns()]
    if (!cols[i]) return
    cols[i] = { ...cols[i], ...patch }
    setTableColumns(cols)
  }

  // dnd-kit generics are incompatible between @dnd-kit/abstract and @dnd-kit-svelte/svelte;
  // the event is only forwarded to `move()`, which handles it correctly at runtime.
  function reorderColumnsByDndEvent(e: unknown) {
    const cols = [...tableColumns()]
    if (cols.length <= 1) return

    const ids = cols.map((c) => c.fieldName)
    const nextIds = move(ids, e as Parameters<typeof move>[1])
    if (nextIds === ids) return

    const byId = new Map(cols.map((c) => [c.fieldName, c] as const))
    const nextCols = nextIds
      .map((id) => byId.get(id))
      .filter((v): v is SqlQueryVizTableColumn => v != null)

    if (nextCols.length === cols.length) {
      setTableColumns(nextCols)
    }
  }

  const scalarSelectValue = $derived(
    vizConfig.scalarField &&
      effectiveFields.some((f) => f.fieldName === vizConfig.scalarField)
      ? vizConfig.scalarField!
      : (effectiveFields[0]?.fieldName ?? '')
  )
</script>

<Sheet.Root
  bind:open
  onOpenChange={(v) => {
    if (v) {
      vizConfig = normalizeVizConfig(vizConfig, isCollection, effectiveFields)
    }
  }}
>
  <Sheet.Portal>
    <Sheet.Overlay class="z-[60]" />
  </Sheet.Portal>
  <Sheet.Content
    class="z-[60] flex w-full flex-col px-4 pt-10 pb-4 sm:max-w-md sm:px-6 sm:pb-6"
  >
    <Sheet.Header class="space-y-1.5 pr-8 text-start">
      <Sheet.Title class="flex items-center gap-2">
        <Settings2 class="size-5 shrink-0" />
        可视化配置
      </Sheet.Title>
      <Sheet.Description>
        配置表格列或标量卡片展示；设置将随查询保存。
      </Sheet.Description>
    </Sheet.Header>

    <div class="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto py-1">
      {#if effectiveFields.length === 0}
        <p class="text-muted-foreground text-sm">
          暂无结果字段元数据。请先保存查询并执行一次（或从列表执行），以同步字段列表；临时预览也可先执行
          SQL。
        </p>
      {:else if isCollection === 0}
        <div class="space-y-2">
          <Label class="text-sm font-medium">卡片显示字段</Label>
          <Select.Root
            type="single"
            value={scalarSelectValue}
            onValueChange={(v) => {
              vizConfig = { version: 1, scalarField: v }
            }}
          >
            <Select.Trigger class="w-full">
              {scalarSelectValue || '选择字段'}
            </Select.Trigger>
            <Select.Content class="z-[61]">
              {#each effectiveFields as f (f.fieldName)}
                <Select.Item value={f.fieldName} label={f.fieldName}>
                  <span class="font-mono text-sm">{f.fieldName}</span>
                  {#if f.commentLabel}
                    <span class="text-muted-foreground ml-2 text-xs"
                      >{f.commentLabel}</span
                    >
                  {/if}
                </Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </div>
      {:else}
        <div class="flex items-center gap-3 text-sm">
          <span class="font-medium">渲染形态</span>
          <label class="flex items-center gap-1">
            <input
              type="radio"
              name="viz-kind"
              checked={vizConfig.kind !== 'chart'}
              onchange={() => switchKind('table')}
            />
            表格
          </label>
          <label class="flex items-center gap-1">
            <input
              type="radio"
              name="viz-kind"
              checked={vizConfig.kind === 'chart'}
              onchange={() => switchKind('chart')}
            />
            图表
          </label>
        </div>

        {#if vizConfig.kind === 'chart' && vizConfig.chart}
          {@const chart = vizConfig.chart}
          <div class="space-y-2">
            <Label class="text-sm font-medium">X 轴字段</Label>
            <Select.Root
              type="single"
              value={chart.xField}
              onValueChange={(v) => updateChartXField(v)}
            >
              <Select.Trigger class="w-full">
                {chart.xField || '选择字段'}
              </Select.Trigger>
              <Select.Content class="z-[61]">
                {#each effectiveFields as f (f.fieldName)}
                  <Select.Item value={f.fieldName} label={f.fieldName}>
                    <span class="font-mono text-sm">{f.fieldName}</span>
                    {#if f.commentLabel}
                      <span class="text-muted-foreground ml-2 text-xs"
                        >{f.commentLabel}</span
                      >
                    {/if}
                  </Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
          </div>

          <div class="space-y-2">
            <p class="text-sm font-medium">系列</p>
            <p class="text-muted-foreground text-xs">
              选择 Y 轴字段与渲染类型(线 / 面积);可添加多条共享 X 轴。
            </p>
            <ul class="space-y-1.5">
              {#each chart.series as s, i (s.fieldName + ':' + i)}
                <li
                  class="bg-muted/40 flex items-center gap-2 rounded-md border px-2 py-2"
                >
                  <Select.Root
                    type="single"
                    value={s.fieldName}
                    onValueChange={(v) =>
                      updateChartSeries(i, { fieldName: v })}
                  >
                    <Select.Trigger class="h-8 w-32">
                      {s.fieldName}
                    </Select.Trigger>
                    <Select.Content class="z-[61]">
                      {#each effectiveFields.filter((f) => f.fieldName !== chart.xField) as f (f.fieldName)}
                        <Select.Item value={f.fieldName} label={f.fieldName}>
                          <span class="font-mono text-xs">{f.fieldName}</span>
                        </Select.Item>
                      {/each}
                    </Select.Content>
                  </Select.Root>

                  <Select.Root
                    type="single"
                    value={s.type}
                    onValueChange={(v) => {
                      if (v === 'line' || v === 'area')
                        updateChartSeries(i, { type: v })
                    }}
                  >
                    <Select.Trigger class="h-8 w-20">
                      {s.type === 'area' ? '面积' : '线'}
                    </Select.Trigger>
                    <Select.Content class="z-[61]">
                      <Select.Item value="line">线</Select.Item>
                      <Select.Item value="area">面积</Select.Item>
                    </Select.Content>
                  </Select.Root>

                  <Input
                    value={s.label ?? ''}
                    placeholder="图例文字"
                    class="h-8 flex-1"
                    oninput={(e) =>
                      updateChartSeries(i, {
                        label: e.currentTarget.value || undefined,
                      })}
                  />

                  <Button
                    variant="ghost"
                    size="icon"
                    class="size-8 shrink-0"
                    type="button"
                    aria-label="删除系列"
                    title="删除"
                    onclick={() => removeChartSeries(i)}
                  >
                    <Trash2 class="size-4" />
                  </Button>
                </li>
              {/each}
            </ul>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onclick={addChartSeries}
            >
              + 添加系列
            </Button>
          </div>
        {:else}
          {@const cols = tableColumns()}
          <label
            class="text-muted-foreground flex cursor-pointer items-center gap-2 text-sm"
          >
            <Checkbox
              checked={vizConfig.table?.showRowIndex === true}
              onCheckedChange={(v) => toggleShowRowIndex(v === true)}
            />
            <span>显示行索引</span>
          </label>

          <div class="space-y-2">
            <p class="text-sm font-medium">字段顺序与可见性</p>
            <p class="text-muted-foreground text-xs">
              拖动排序；眼睛切换显示；「···」设置标题与对齐。
            </p>
            <DragDropProvider
              modifiers={[
                // @ts-expect-error — RestrictToVerticalAxis modifier type is incompatible with DragDropProvider generics (dnd-kit type limitation)
                RestrictToVerticalAxis,
              ]}
              onDragEnd={reorderColumnsByDndEvent}
            >
              <ul class="space-y-1.5">
                {#each cols as col, i (col.fieldName)}
                  {@const { ref, isDragging, handleRef } = useSortable({
                    id: col.fieldName,
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
                    <span
                      class="bg-background text-muted-foreground flex size-7 shrink-0 items-center justify-center rounded border font-mono text-xs font-bold"
                      title={col.fieldName}
                    >
                      {isNumericFieldType(
                        effectiveFields.find(
                          (f) => f.fieldName === col.fieldName
                        )?.fieldType
                      )
                        ? '#'
                        : 'T'}
                    </span>
                    <span class="min-w-0 flex-1 truncate font-mono text-sm">
                      {col.fieldName}
                    </span>

                    <Popover.Root>
                      <Popover.Trigger>
                        {#snippet child({ props })}
                          <Button
                            variant="ghost"
                            size="icon"
                            class="size-8 shrink-0"
                            type="button"
                            aria-label="列设置"
                            {...props}
                          >
                            <MoreHorizontal class="size-4" />
                          </Button>
                        {/snippet}
                      </Popover.Trigger>
                      <Popover.Content
                        class="z-[61] w-72 space-y-3 p-3"
                        align="end"
                      >
                        <div class="space-y-1.5">
                          <Label for="viz-title-{i}" class="text-xs"
                            >字段标题</Label
                          >
                          <Input
                            id="viz-title-{i}"
                            value={col.title ?? col.fieldName}
                            oninput={(e) =>
                              updateColumn(i, {
                                title: e.currentTarget.value || undefined,
                              })}
                            placeholder={col.fieldName}
                          />
                        </div>
                        <div class="space-y-1.5">
                          <span class="text-xs font-medium">对齐</span>
                          <Select.Root
                            type="single"
                            value={col.align}
                            onValueChange={(v) => {
                              if (
                                v === 'left' ||
                                v === 'center' ||
                                v === 'right'
                              ) {
                                updateColumn(i, { align: v })
                              }
                            }}
                          >
                            <Select.Trigger class="w-full">
                              {col.align === 'center'
                                ? '居中'
                                : col.align === 'right'
                                  ? '右对齐'
                                  : '左对齐'}
                            </Select.Trigger>
                            <Select.Content class="z-[61]">
                              <Select.Item value="left">左对齐</Select.Item>
                              <Select.Item value="center">居中</Select.Item>
                              <Select.Item value="right">右对齐</Select.Item>
                            </Select.Content>
                          </Select.Root>
                        </div>
                      </Popover.Content>
                    </Popover.Root>

                    <Button
                      variant="ghost"
                      size="icon"
                      class={cn(
                        'size-8 shrink-0',
                        !col.visible && 'opacity-50'
                      )}
                      type="button"
                      onclick={() => toggleColumnVisible(i)}
                      aria-label={col.visible ? '隐藏列' : '显示列'}
                      title={col.visible ? '隐藏' : '显示'}
                    >
                      {#if col.visible}
                        <Eye class="size-4" />
                      {:else}
                        <EyeOff class="size-4" />
                      {/if}
                    </Button>
                  </li>
                {/each}
              </ul>
            </DragDropProvider>
          </div>
        {/if}
      {/if}
    </div>

    <Sheet.Footer
      class="border-border -mx-4 flex flex-row flex-wrap items-center justify-end gap-2 border-t px-4 pt-4 sm:-mx-6 sm:px-6"
    >
      <Button
        type="button"
        variant="outline"
        disabled={saving}
        onclick={handleClose}
      >
        关闭
      </Button>
      {#if onVizConfigPersist}
        <Button
          type="button"
          variant="default"
          disabled={saving}
          onclick={handleSave}
        >
          {saving ? '保存中…' : '保存'}
        </Button>
      {/if}
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>
