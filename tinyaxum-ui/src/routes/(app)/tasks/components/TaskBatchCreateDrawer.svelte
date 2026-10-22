<script lang="ts">
  import { isApiClientNotifiedError } from '$lib/api/client.js'
  import { datasourcesApi } from '$lib/api/datasources.js'
  import { transtaskApi } from '$lib/api/transtask.js'
  import type { DatasourceTableSchemaVo } from '$lib/api/types.js'
  import { Button } from '$lib/components/ui/button/index.js'
  import { Checkbox } from '$lib/components/ui/checkbox/index.js'
  import * as Field from '$lib/components/ui/field/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import * as Popover from '$lib/components/ui/popover/index.js'
  import * as Sheet from '$lib/components/ui/sheet/index.js'
  import { ChevronsUpDown, LoaderCircle, Search } from '@lucide/svelte'
  import { toast } from 'svelte-sonner'
  import {
    TRANS_TASK_BATCH_MAX_TABLES,
    transTaskBatchCreateRequestSchema,
  } from '../data/schema.js'
  import { useTasks } from './tasks-state.svelte'

  const tasks = useTasks()

  const SYNC_MODES = [
    { value: 'full', label: '全量同步' },
    { value: 'incremental', label: '增量同步' },
    { value: 'realtime', label: '实时同步' },
  ] as const

  const SYNC_STRATEGIES = [
    { value: 'timestamp', label: '时间戳 (有更新的表)' },
    { value: 'auto_increment', label: '自增ID (仅追加的表)' },
  ] as const

  type BatchForm = {
    dataSourceId: number
    taskCron: string
    remark: string
    syncMode: string
    syncStrategy: string
    syncColumn: string
    syncPrimaryKey: string
  }

  const defaultForm = (): BatchForm => ({
    dataSourceId: 0,
    taskCron: '',
    remark: '',
    syncMode: 'full',
    syncStrategy: '',
    syncColumn: '',
    syncPrimaryKey: '',
  })

  let formData = $state<BatchForm>(defaultForm())
  const datasources = $derived(tasks.datasources)
  let schemaTables = $state<DatasourceTableSchemaVo[]>([])
  let schemaLoading = $state(false)
  let schemaError = $state<string | null>(null)
  let selectedNames = $state<Set<string>>(new Set())
  let tablesPopoverOpen = $state(false)
  let tableFilterQuery = $state('')
  let errors = $state<Record<string, string[]>>({})
  let isSubmitting = $state(false)
  let lastBatchOpen = $state(false)

  const needsIncrementalFields = $derived(
    formData.syncMode === 'incremental' || formData.syncMode === 'realtime'
  )
  const needsPrimaryKey = $derived(formData.syncStrategy === 'timestamp')

  function rowComment(t: DatasourceTableSchemaVo): string {
    const a = (t.commentLabel ?? '').trim()
    const b = (t.tableDesc ?? '').trim()
    return a || b || t.tableName
  }

  $effect(() => {
    const open = tasks.open === 'batchCreate'
    if (open && !lastBatchOpen) {
      formData = {
        dataSourceId: datasources[0]?.dataSourceId ?? 0,
        taskCron: '',
        remark: '',
        syncMode: 'full',
        syncStrategy: '',
        syncColumn: '',
        syncPrimaryKey: '',
      }
      selectedNames = new Set()
      tablesPopoverOpen = false
      errors = {}
    }
    lastBatchOpen = open
  })

  $effect(() => {
    if (tasks.open !== 'batchCreate') return
    if (formData.dataSourceId === 0 && datasources[0]) {
      formData = { ...formData, dataSourceId: datasources[0].dataSourceId }
    }
  })

  $effect(() => {
    if (tasks.open !== 'batchCreate') return
    const id = formData.dataSourceId
    if (id < 1) {
      schemaTables = []
      schemaError = null
      schemaLoading = false
      return
    }

    let cancelled = false
    schemaLoading = true
    schemaError = null
    selectedNames = new Set()

    datasourcesApi
      .getSchema(id)
      .then((res) => {
        if (cancelled) return
        schemaTables = res.data?.tables ?? []
      })
      .catch((e) => {
        if (cancelled) return
        schemaError = e instanceof Error ? e.message : '加载表列表失败'
        schemaTables = []
      })
      .finally(() => {
        if (!cancelled) schemaLoading = false
      })

    return () => {
      cancelled = true
    }
  })

  $effect(() => {
    if (!tablesPopoverOpen) {
      tableFilterQuery = ''
    }
  })

  const filteredSchemaTables = $derived.by(() => {
    const q = tableFilterQuery.trim().toLowerCase()
    if (q === '') return schemaTables
    return schemaTables.filter((t) => t.tableName.toLowerCase().includes(q))
  })

  function toggleTable(name: string, checked: boolean) {
    const next = new Set(selectedNames)
    if (checked) {
      if (next.size >= TRANS_TASK_BATCH_MAX_TABLES) {
        toast.error(`一次最多选择 ${TRANS_TASK_BATCH_MAX_TABLES} 张表`)
        return
      }
      next.add(name)
    } else {
      next.delete(name)
    }
    selectedNames = next
  }

  function selectAllTables() {
    const source = filteredSchemaTables
    const next = new Set<string>()
    for (const t of source) {
      if (next.size >= TRANS_TASK_BATCH_MAX_TABLES) break
      next.add(t.tableName)
    }
    selectedNames = next
    if (source.length > TRANS_TASK_BATCH_MAX_TABLES) {
      toast.message(`仅选中前 ${TRANS_TASK_BATCH_MAX_TABLES} 张表`)
    }
  }

  function clearTableSelection() {
    selectedNames = new Set()
  }

  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault()
    errors = {}

    const tables = [...selectedNames]
      .map((name) => {
        const t = schemaTables.find((x) => x.tableName === name)
        if (!t) return null
        return {
          tableName: t.tableName,
          tableComment: rowComment(t),
        }
      })
      .filter(
        (x): x is { tableName: string; tableComment: string } => x !== null
      )

    const remarkTrimmed = formData.remark.trim()
    const payload = {
      dataSourceId: formData.dataSourceId,
      taskCron: formData.taskCron.trim(),
      remark: remarkTrimmed === '' ? undefined : remarkTrimmed,
      taskStatus: 'stop',
      tables,
      syncMode: formData.syncMode || undefined,
      syncStrategy: formData.syncStrategy || undefined,
      syncColumn: formData.syncColumn.trim() || undefined,
      syncPrimaryKey: formData.syncPrimaryKey.trim() || undefined,
    }

    const parsed = transTaskBatchCreateRequestSchema.safeParse(payload)
    if (!parsed.success) {
      errors = parsed.error.flatten().fieldErrors as Record<string, string[]>
      return
    }

    isSubmitting = true
    try {
      await transtaskApi.batchCreate(parsed.data)
      toast.success(`已批量创建 ${parsed.data.tables.length} 条任务`)
      tasks.setOpen(null)
      tasks.requestListRefetch()
    } catch (error) {
      const message = error instanceof Error ? error.message : '批量保存失败'
      if (isApiClientNotifiedError(error)) {
        errors = {}
      } else {
        errors = { _form: [message] }
        toast.error(message)
      }
    } finally {
      isSubmitting = false
    }
  }

  function handleOpenChange(open: boolean) {
    if (!open) {
      tasks.setOpen(null)
      setTimeout(() => {
        tasks.setCurrentRow(null)
      }, 200)
    }
  }

  const selectedCount = $derived(selectedNames.size)
  const hasTablesLoaded = $derived(
    !schemaLoading && schemaTables.length > 0 && !schemaError
  )
</script>

<Sheet.Root open={tasks.open === 'batchCreate'} onOpenChange={handleOpenChange}>
  <Sheet.Content class="flex flex-col">
    <Sheet.Header class="text-start">
      <Sheet.Title>批量创建任务</Sheet.Title>
      <Sheet.Description>
        选择数据源与多张表，统一配置 Cron
        与备注后一次提交。表列表来自已缓存的元数据；若为空请在数据源中刷新
        schema。
      </Sheet.Description>
    </Sheet.Header>

    <form
      id="tasks-batch-form"
      onsubmit={handleSubmit}
      class="flex-1 space-y-4 overflow-y-auto px-4 py-4"
    >
      <Field.Field>
        <Field.Label>数据源</Field.Label>
        <select
          bind:value={formData.dataSourceId}
          class="border-input bg-background ring-offset-background focus-visible:ring-ring flex h-10 w-full rounded-md border px-3 py-2 text-sm focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
          disabled={isSubmitting}
        >
          <option value={0} disabled>选择数据源</option>
          {#each datasources as datasource}
            <option value={datasource.dataSourceId}>
              {datasource.name} ({datasource.dbType})
            </option>
          {/each}
        </select>
        <Field.Error
          errors={errors.dataSourceId?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label
          >表（可多选，最多 {TRANS_TASK_BATCH_MAX_TABLES} 张）</Field.Label
        >
        <Popover.Root bind:open={tablesPopoverOpen}>
          <Popover.Trigger>
            {#snippet child({ props })}
              <Button
                variant="outline"
                type="button"
                class="h-10 w-full justify-between font-normal"
                disabled={isSubmitting ||
                  schemaLoading ||
                  formData.dataSourceId < 1}
                {...props}
              >
                <span class="truncate text-left">
                  {#if schemaLoading}
                    加载表列表…
                  {:else if selectedCount > 0}
                    已选 {selectedCount} 张表
                  {:else}
                    点击选择表
                  {/if}
                </span>
                <ChevronsUpDown class="ml-2 size-4 shrink-0 opacity-50" />
              </Button>
            {/snippet}
          </Popover.Trigger>
          <Popover.Content class="w-[min(100vw-3rem,26rem)] p-0" align="start">
            <div
              class="flex items-center justify-between gap-2 border-b px-2 py-2"
            >
              <Button
                type="button"
                variant="ghost"
                size="sm"
                class="h-8"
                onclick={selectAllTables}
                disabled={!hasTablesLoaded || filteredSchemaTables.length === 0}
              >
                全选
              </Button>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                class="h-8"
                onclick={clearTableSelection}
                disabled={selectedCount === 0}
              >
                清空
              </Button>
            </div>
            {#if hasTablesLoaded}
              <div class="border-b px-2 pb-2">
                <div class="relative">
                  <Search
                    class="text-muted-foreground pointer-events-none absolute top-1/2 left-2 size-4 -translate-y-1/2"
                  />
                  <Input
                    bind:value={tableFilterQuery}
                    type="search"
                    placeholder="过滤表名…"
                    class="h-9 pl-8"
                    autocomplete="off"
                  />
                </div>
              </div>
            {/if}
            <div class="max-h-64 overflow-y-auto p-2">
              {#if schemaError}
                <p class="text-destructive text-sm">{schemaError}</p>
              {:else if !schemaLoading && schemaTables.length === 0}
                <p class="text-muted-foreground text-sm">
                  暂无表元数据，请在数据源管理中刷新 schema。
                </p>
              {:else if hasTablesLoaded && filteredSchemaTables.length === 0}
                <p class="text-muted-foreground text-sm">无匹配的表名</p>
              {:else}
                {#each filteredSchemaTables as t (t.tableMetaId)}
                  <label
                    class="hover:bg-muted/50 flex cursor-pointer items-start gap-2 rounded-md px-1 py-1.5"
                  >
                    <Checkbox
                      class="mt-0.5"
                      checked={selectedNames.has(t.tableName)}
                      onCheckedChange={(v) =>
                        toggleTable(t.tableName, v === true)}
                    />
                    <span class="min-w-0 flex-1">
                      <span class="block truncate font-medium"
                        >{t.tableName}</span
                      >
                      {#if rowComment(t) !== t.tableName}
                        <span
                          class="text-muted-foreground block truncate text-xs"
                          >{rowComment(t)}</span
                        >
                      {/if}
                    </span>
                  </label>
                {/each}
              {/if}
            </div>
          </Popover.Content>
        </Popover.Root>
        <Field.Error
          errors={errors.tables?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>Cron表达式</Field.Label>
        <Input
          bind:value={formData.taskCron}
          placeholder="例如：0 */5 * * * *"
          disabled={isSubmitting}
        />
        <Field.Error
          errors={errors.taskCron?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>同步模式</Field.Label>
        <select
          bind:value={formData.syncMode}
          class="border-input bg-background ring-offset-background focus-visible:ring-ring flex h-10 w-full rounded-md border px-3 py-2 text-sm focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
          disabled={isSubmitting}
        >
          {#each SYNC_MODES as mode}
            <option value={mode.value}>{mode.label}</option>
          {/each}
        </select>
        <Field.Description>
          {#if formData.syncMode === 'full'}
            每次同步全量拉取表数据
          {:else if formData.syncMode === 'incremental'}
            仅拉取自上次同步后变化的数据
          {:else}
            高频增量轮询（建议 Cron 设为 */10 * * * * *）
          {/if}
        </Field.Description>
      </Field.Field>

      {#if needsIncrementalFields}
        <Field.Field>
          <Field.Label>增量策略</Field.Label>
          <select
            bind:value={formData.syncStrategy}
            class="border-input bg-background ring-offset-background focus-visible:ring-ring flex h-10 w-full rounded-md border px-3 py-2 text-sm focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
            disabled={isSubmitting}
          >
            <option value="" disabled>选择增量策略</option>
            {#each SYNC_STRATEGIES as strategy}
              <option value={strategy.value}>{strategy.label}</option>
            {/each}
          </select>
          <Field.Error
            errors={errors.syncStrategy?.map((m) => ({ message: m })) ?? []}
          />
        </Field.Field>

        <Field.Field>
          <Field.Label>增量列名</Field.Label>
          <Input
            bind:value={formData.syncColumn}
            placeholder="例如：updated_at 或 id"
            disabled={isSubmitting}
          />
          <Field.Description>
            源表中用于判断数据变化的列（所有选中表需有此列）
          </Field.Description>
          <Field.Error
            errors={errors.syncColumn?.map((m) => ({ message: m })) ?? []}
          />
        </Field.Field>

        {#if needsPrimaryKey}
          <Field.Field>
            <Field.Label>主键列名</Field.Label>
            <Input
              bind:value={formData.syncPrimaryKey}
              placeholder="例如：id"
              disabled={isSubmitting}
            />
            <Field.Description>
              用于合并去重的主键列（timestamp 策略必填）
            </Field.Description>
            <Field.Error
              errors={errors.syncPrimaryKey?.map((m) => ({ message: m })) ?? []}
            />
          </Field.Field>
        {/if}
      {/if}

      <Field.Field>
        <Field.Label>备注</Field.Label>
        <Input
          bind:value={formData.remark}
          placeholder="可选备注（应用于每条任务）"
          disabled={isSubmitting}
        />
        <Field.Error
          errors={errors.remark?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      {#if errors._form}
        <div class="text-destructive text-sm">{errors._form.join(', ')}</div>
      {/if}
    </form>

    <Sheet.Footer class="flex flex-row justify-end gap-2">
      <Sheet.Close>
        <Button variant="outline" disabled={isSubmitting}>关闭</Button>
      </Sheet.Close>
      <Button form="tasks-batch-form" type="submit" disabled={isSubmitting}>
        {#if isSubmitting}
          <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
        {/if}
        保存
      </Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>
