<script lang="ts">
  import { untrack } from 'svelte'
  import { isApiClientNotifiedError } from '$lib/api/client.js'
  import { datasourcesApi } from '$lib/api/datasources.js'
  import { sqlQueriesApi } from '$lib/api/sqlqueries.js'
  import type { DatasourceListVo, SqlQueryListVo } from '$lib/api/types.js'
  import DatasourceFilter from '$lib/components/crud/DatasourceFilter.svelte'
  import { Button } from '$lib/components/ui/button/index.js'
  import { Checkbox } from '$lib/components/ui/checkbox/index.js'
  import * as Collapsible from '$lib/components/ui/collapsible/index.js'
  import * as Command from '$lib/components/ui/command/index.js'
  import * as Field from '$lib/components/ui/field/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import * as Popover from '$lib/components/ui/popover/index.js'
  import * as Sheet from '$lib/components/ui/sheet/index.js'
  import {
    httpApiCreateSchema,
    httpApiUpdateSchema,
    type HttpApi,
  } from '$lib/data/httpapi/schema.js'
  import { copyText } from '$lib/utils/clipboard.js'
  import {
    ChevronDown,
    CircleHelp,
    Copy,
    LoaderCircle,
    Pencil,
    Plus,
    Trash2,
  } from '@lucide/svelte'
  import SearchIcon from '@lucide/svelte/icons/search'
  import { Command as CommandPrimitive } from 'bits-ui'
  import { toast } from 'svelte-sonner'
  import { z } from 'zod'
  import { useHttpApi } from './httpapi-state.svelte'

  const apiState = useHttpApi()

  type QueryItem = {
    sqlQueryId: number
    queryAlias: string
    sortOrder: number
    sqlQueryName?: string
  }

  interface FormData {
    httpApiName: string
    httpApiPath: string
    httpApiDesc: string
    module: string
    category: string
    enabled: number
    responseTemplate: string
    defaultParamsText: string
    queries: QueryItem[]
  }

  let formData: FormData = $state({
    httpApiName: '',
    httpApiPath: '',
    httpApiDesc: '',
    module: '',
    category: '',
    enabled: 1,
    responseTemplate: '',
    defaultParamsText: '',
    queries: [],
  })

  let errors: Record<string, string[]> = $state({})
  let isSubmitting = $state(false)
  let isLoadingDetail = $state(false)

  let sqlQueries: SqlQueryListVo[] = $state([])
  let datasources: DatasourceListVo[] = $state([])
  let selectedSqlQueryId: string = $state('')
  let newAlias: string = $state('')
  let popoverOpen = $state(false)
  let dsFilter = $state<number | null>(null)
  let helpOpen = $state(false)

  const isEdit = $derived(apiState.open === 'edit')
  const isCopy = $derived(apiState.open === 'copy')

  const dsNameById = $derived(
    new Map(datasources.map((d) => [d.dataSourceId, d.name]))
  )

  const filteredQueries = $derived(
    sqlQueries.filter((q) => dsFilter == null || q.dataSourceId === dsFilter)
  )

  const selectedQueryName = $derived.by(() => {
    const id = Number(selectedSqlQueryId)
    if (!id) return ''
    const q = sqlQueries.find((s) => s.sqlQueryId === id)
    return q?.sqlQueryName ?? ''
  })

  $effect(() => {
    if (!popoverOpen) {
      dsFilter = null
    }
  })

  function resetForm() {
    formData = {
      httpApiName: '',
      httpApiPath: '',
      httpApiDesc: '',
      module: '',
      category: '',
      enabled: 1,
      responseTemplate: '',
      defaultParamsText: '',
      queries: [],
    }
    errors = {}
    selectedSqlQueryId = ''
    newAlias = ''
  }

  async function loadSqlQueries() {
    try {
      const res = await sqlQueriesApi.list()
      if (res.data) {
        sqlQueries = res.data
      }
    } catch {
      toast.error('加载SQL查询列表失败')
    }
    try {
      const dres = await datasourcesApi.list()
      datasources = Array.isArray(dres.data) ? dres.data : []
    } catch {
      datasources = []
    }
  }

  function detailToFormData(detail: {
    httpApiName: string
    httpApiPath: string
    httpApiDesc?: string | null
    module?: string | null
    category?: string | null
    enabled: number
    responseTemplate?: string | null
    defaultParams?: Record<string, string | number | boolean> | null
    queries: {
      sqlQueryId: number
      queryAlias: string
      sortOrder: number
      sqlQueryName?: string
    }[]
  }): FormData {
    return {
      httpApiName: detail.httpApiName,
      httpApiPath: detail.httpApiPath,
      httpApiDesc: detail.httpApiDesc ?? '',
      module: detail.module ?? '',
      category: detail.category ?? '',
      enabled: detail.enabled,
      responseTemplate: detail.responseTemplate ?? '',
      defaultParamsText: detail.defaultParams
        ? JSON.stringify(detail.defaultParams, null, 2)
        : '',
      queries: detail.queries.map((q) => ({
        sqlQueryId: q.sqlQueryId,
        queryAlias: q.queryAlias,
        sortOrder: q.sortOrder,
        sqlQueryName: q.sqlQueryName,
      })),
    }
  }

  async function loadDetail(id: number) {
    isLoadingDetail = true
    try {
      const detail = await apiState.loadDetail(id)
      if (detail) {
        untrack(() => {
          formData = detailToFormData(detail)
        })
      }
    } catch (e) {
      const message = e instanceof Error ? e.message : '加载详情失败'
      if (!isApiClientNotifiedError(e)) {
        errors = { _form: [message] }
        toast.error(message)
      }
    } finally {
      isLoadingDetail = false
    }
  }

  $effect(() => {
    if (apiState.open === 'edit' && apiState.currentRow) {
      const api = apiState.currentRow as HttpApi
      void loadDetail(api.httpApiId)
    } else if (apiState.open === 'copy' && apiState.copyApiData) {
      const detail = apiState.copyApiData
      untrack(() => {
        formData = detailToFormData(detail)
      })
    } else if (apiState.open === 'create') {
      untrack(() => resetForm())
    }
    errors = {}
    void loadSqlQueries()
  })

  function addQuery() {
    const id = Number(selectedSqlQueryId)
    if (!id) {
      toast.error('请选择SQL查询')
      return
    }
    if (!newAlias.trim()) {
      toast.error('请输入别名')
      return
    }
    const dup = formData.queries.find(
      (q: QueryItem) => q.queryAlias === newAlias.trim()
    )
    if (dup) {
      toast.error('别名已存在')
      return
    }
    const sq = sqlQueries.find((s: SqlQueryListVo) => s.sqlQueryId === id)
    formData.queries = [
      ...formData.queries,
      {
        sqlQueryId: id,
        queryAlias: newAlias.trim(),
        sortOrder: formData.queries.length,
        sqlQueryName: sq?.sqlQueryName,
      },
    ]
    selectedSqlQueryId = ''
    newAlias = ''
  }

  function removeQuery(index: number) {
    formData.queries = formData.queries.filter(
      (_: QueryItem, i: number) => i !== index
    )
  }

  function handleCopyPath() {
    const path = `/api/public/${formData.httpApiPath}`
    void copyText(path).then(() => {
      toast.success('已复制路径: ' + path)
    })
  }

  function generateDefaultTemplate() {
    if (formData.queries.length === 0) {
      toast.error('请先关联SQL查询')
      return
    }
    const pairs = formData.queries.map((q) => {
      const sq = sqlQueries.find((s) => s.sqlQueryId === q.sqlQueryId)
      if (sq && sq.isCollection === 1) {
        return `  "${q.queryAlias}": {\n    "rows": {{${q.queryAlias}}},\n    "total": {{${q.queryAlias}_total}},\n    "current": {{${q.queryAlias}_current}},\n    "size": {{${q.queryAlias}_size}},\n    "pages": {{${q.queryAlias}_pages}}\n  }`
      }
      return `  "${q.queryAlias}": {{${q.queryAlias}}}`
    })
    formData.responseTemplate = `{\n${pairs.join(',\n')}\n}`
  }

  function toCreateInput(
    data: FormData
  ):
    | { ok: true; value: z.infer<typeof httpApiCreateSchema> }
    | { ok: false; message: string } {
    let defaultParams: Record<string, string> | undefined
    const text = data.defaultParamsText.trim()
    if (text) {
      let parsed: unknown
      try {
        parsed = JSON.parse(text)
      } catch {
        return { ok: false, message: '固定参数不是合法的 JSON' }
      }
      if (
        typeof parsed !== 'object' ||
        parsed === null ||
        Array.isArray(parsed)
      ) {
        return { ok: false, message: '固定参数必须是 JSON 对象' }
      }
      defaultParams = Object.fromEntries(
        Object.entries(parsed as Record<string, unknown>).map(([k, v]) => [
          k,
          String(v),
        ])
      )
    }
    return {
      ok: true,
      value: {
        httpApiName: data.httpApiName,
        httpApiPath: data.httpApiPath,
        httpApiDesc: data.httpApiDesc || undefined,
        module: data.module || undefined,
        category: data.category || undefined,
        enabled: data.enabled,
        responseTemplate: data.responseTemplate || undefined,
        defaultParams,
        queries: data.queries.map((q) => ({
          sqlQueryId: q.sqlQueryId,
          queryAlias: q.queryAlias,
          sortOrder: q.sortOrder,
        })),
      },
    }
  }

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault()

    const input = toCreateInput(formData)
    if (!input.ok) {
      errors = { defaultParams: [input.message] }
      toast.error(input.message)
      return
    }
    const schemaToUse = isEdit ? httpApiUpdateSchema : httpApiCreateSchema
    const payload = isEdit
      ? {
          ...input.value,
          httpApiId: (apiState.currentRow as HttpApi).httpApiId,
        }
      : input.value

    const parseResult = schemaToUse.safeParse(payload)

    if (!parseResult.success) {
      errors = parseResult.error.flatten().fieldErrors as Record<
        string,
        string[]
      >
      return
    }

    errors = {}
    isSubmitting = true

    try {
      if (isEdit) {
        const ok = await apiState.updateApi(
          parseResult.data as z.infer<typeof httpApiUpdateSchema>
        )
        if (!ok) return
      } else {
        // create or copy
        const ok = await apiState.createApi(parseResult.data)
        if (!ok) return
      }
      apiState.closeDialogs()
      apiState.resetCopyApi()
      apiState.requestListRefetch()
    } catch (e) {
      const message =
        e instanceof Error ? e.message : isEdit ? '更新失败' : '创建失败'
      if (!isApiClientNotifiedError(e)) {
        errors = { _form: [message] }
        toast.error(message)
      }
    } finally {
      isSubmitting = false
    }
  }

  function handleOpenChange(open: boolean) {
    if (!open) {
      apiState.closeDialogs()
      apiState.resetCopyApi()
    }
  }
</script>

<Sheet.Root
  open={apiState.open === 'create' ||
    apiState.open === 'edit' ||
    apiState.open === 'copy'}
  onOpenChange={handleOpenChange}
>
  <Sheet.Content class="flex flex-col">
    <Sheet.Header class="text-start">
      <Sheet.Title
        >{isEdit ? '编辑API' : isCopy ? '复制API' : '创建API'}</Sheet.Title
      >
      <Sheet.Description>
        {isEdit
          ? '更新HTTP API配置。'
          : isCopy
            ? '基于现有API创建副本。'
            : '配置新的HTTP API接口。'}
      </Sheet.Description>
    </Sheet.Header>

    <form
      id="httpapi-form"
      onsubmit={handleSubmit}
      class="flex-1 space-y-4 overflow-y-auto px-4 py-4"
    >
      <Field.Field>
        <Field.Label>API名称</Field.Label>
        <Input
          bind:value={formData.httpApiName}
          placeholder="请输入API名称"
          disabled={isSubmitting || isLoadingDetail}
        />
        <Field.Error
          errors={errors.httpApiName?.map((m: string) => ({
            message: m,
          })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>访问路径</Field.Label>
        <div class="flex items-center gap-0">
          <span
            class="border-input bg-muted inline-flex h-10 items-center rounded-l-md border border-r-0 px-3 text-sm text-nowrap"
          >
            /api/public/
          </span>
          <Input
            bind:value={formData.httpApiPath}
            placeholder="your-endpoint"
            class="rounded-l-none"
            disabled={isSubmitting || isLoadingDetail}
          />
          <Button
            type="button"
            variant="outline"
            size="icon"
            class="ml-2 size-10 shrink-0"
            title="复制完整路径"
            aria-label="复制完整路径"
            onclick={handleCopyPath}
            disabled={!formData.httpApiPath}
          >
            <Copy class="h-4 w-4" />
          </Button>
        </div>
        <Field.Error
          errors={errors.httpApiPath?.map((m: string) => ({
            message: m,
          })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>描述</Field.Label>
        <textarea
          bind:value={formData.httpApiDesc}
          placeholder="请输入描述（可选）"
          rows={3}
          disabled={isSubmitting || isLoadingDetail}
          class="border-input bg-background ring-offset-background focus-visible:ring-ring flex min-h-[80px] w-full rounded-md border px-3 py-2 text-sm focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
        ></textarea>
        <Field.Error
          errors={errors.httpApiDesc?.map((m: string) => ({
            message: m,
          })) ?? []}
        />
      </Field.Field>

      <div class="grid grid-cols-2 gap-3">
        <Field.Field>
          <Field.Label>功能模块</Field.Label>
          <Input
            bind:value={formData.module}
            placeholder="请输入功能模块（可选）"
            disabled={isSubmitting || isLoadingDetail}
          />
        </Field.Field>
        <Field.Field>
          <Field.Label>分类</Field.Label>
          <Input
            bind:value={formData.category}
            placeholder="请输入分类（可选）"
            disabled={isSubmitting || isLoadingDetail}
          />
        </Field.Field>
      </div>

      <Field.Field>
        <div class="flex items-center gap-2">
          <Checkbox
            checked={formData.enabled === 1}
            onCheckedChange={(v) => {
              formData.enabled = v ? 1 : 0
            }}
            disabled={isSubmitting || isLoadingDetail}
          />
          <span class="text-sm">启用状态</span>
        </div>
      </Field.Field>

      <Field.Field>
        <Field.Label>固定参数（JSON，可选）</Field.Label>
        <textarea
          bind:value={formData.defaultParamsText}
          placeholder={'{\n  "startDays": 2,\n  "endDays": 15,\n  "icd10s": "I21,I63"\n}'}
          rows={4}
          disabled={isSubmitting || isLoadingDetail}
          class="border-input bg-background ring-offset-background focus-visible:ring-ring flex min-h-[90px] w-full rounded-md border px-3 py-2 font-mono text-sm focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
        ></textarea>
        <p class="text-muted-foreground text-xs">
          执行时覆盖同名请求参数，用于锁定查询口径（如再住院天数窗口、ICD
          列表、固定科室等）
        </p>
        <Field.Error
          errors={errors.defaultParams?.map((m: string) => ({
            message: m,
          })) ?? []}
        />
      </Field.Field>

      <div class="space-y-3">
        <div class="text-sm font-medium">SQL查询关联</div>

        <div class="flex items-end gap-2 rounded-md border p-3">
          <div class="flex-1">
            <label
              for="sql-query-popover-trigger"
              class="text-muted-foreground mb-1 block text-xs"
            >
              选择SQL查询
            </label>
            <Popover.Root bind:open={popoverOpen}>
              <Popover.Trigger>
                {#snippet child({ props })}
                  <Button
                    id="sql-query-popover-trigger"
                    {...props}
                    variant="outline"
                    class="w-full justify-between"
                    disabled={isSubmitting || isLoadingDetail}
                  >
                    <span
                      class={selectedQueryName ? '' : 'text-muted-foreground'}
                    >
                      {selectedQueryName || '选择要添加的 SQL 查询…'}
                    </span>
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
                      value={dsFilter}
                      {datasources}
                      placeholder="全部"
                      onChange={(id) => (dsFilter = id)}
                    />
                    <SearchIcon class="size-4 shrink-0 opacity-50" />
                    <CommandPrimitive.Input
                      placeholder="搜索查询名…"
                      class="placeholder:text-muted-foreground h-9 flex-1 bg-transparent text-sm outline-none"
                    />
                  </div>
                  <Command.List class="max-h-[300px]">
                    <Command.Empty>未找到匹配的查询</Command.Empty>
                    {#each filteredQueries as sq (sq.sqlQueryId)}
                      <Command.Item
                        value={`${sq.sqlQueryName} ${dsNameById.get(sq.dataSourceId) ?? ''}`}
                        onSelect={() => {
                          selectedSqlQueryId = String(sq.sqlQueryId)
                          popoverOpen = false
                        }}
                      >
                        <span class="flex-1 truncate">{sq.sqlQueryName}</span>
                        <span class="text-muted-foreground shrink-0 text-xs">
                          · {dsNameById.get(sq.dataSourceId) ??
                            `DS#${sq.dataSourceId}`}
                        </span>
                      </Command.Item>
                    {/each}
                  </Command.List>
                </Command.Root>
              </Popover.Content>
            </Popover.Root>
          </div>
          <div class="flex-1">
            <label
              for="query-alias-input"
              class="text-muted-foreground mb-1 block text-xs"
            >
              别名
            </label>
            <Input
              id="query-alias-input"
              bind:value={newAlias}
              placeholder=""
              disabled={isSubmitting || isLoadingDetail}
              class="h-9"
            />
          </div>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onclick={addQuery}
            disabled={isSubmitting ||
              isLoadingDetail ||
              !selectedSqlQueryId ||
              !newAlias.trim()}
            class="h-9"
          >
            <Plus class="h-4 w-4" />
          </Button>
        </div>

        {#if formData.queries.length > 0}
          <div class="space-y-2">
            {#each formData.queries as q, i}
              <div
                class="bg-muted/50 flex items-center justify-between rounded-md border px-3 py-2 text-sm"
              >
                <div class="flex items-center gap-2">
                  <span class="font-mono text-xs">{q.queryAlias}</span>
                  <span class="text-muted-foreground">
                    &mdash; {q.sqlQueryName ?? `查询#${q.sqlQueryId}`}
                  </span>
                </div>
                <div class="flex items-center gap-1">
                  <a
                    href="/sqlquery?edit={q.sqlQueryId}"
                    aria-label="编辑 SQL 查询"
                    title="编辑 SQL 查询"
                    class="text-muted-foreground hover:text-foreground transition-colors"
                  >
                    <Pencil class="h-4 w-4" />
                  </a>
                  <button
                    type="button"
                    onclick={() => removeQuery(i)}
                    disabled={isSubmitting}
                    class="text-muted-foreground hover:text-destructive transition-colors"
                  >
                    <Trash2 class="h-4 w-4" />
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}

        <Field.Error
          errors={errors.queries?.map((m: string) => ({
            message: m,
          })) ?? []}
        />
      </div>

      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <div class="text-sm font-medium">响应模板</div>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onclick={generateDefaultTemplate}
            disabled={isSubmitting ||
              isLoadingDetail ||
              formData.queries.length === 0}
          >
            生成默认模板
          </Button>
        </div>
        <textarea
          bind:value={formData.responseTemplate}
          placeholder={'{\n  "key": "{{alias}}",\n  "field": "{{alias.field}}",\n  "yoy_rate": "{{changeRate alias.field alias_yoy.field}}"\n}'}
          rows={8}
          disabled={isSubmitting || isLoadingDetail}
          class="border-input bg-background ring-offset-background focus-visible:ring-ring flex min-h-[160px] w-full rounded-md border px-3 py-2 font-mono text-sm focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
        ></textarea>
        <Collapsible.Root bind:open={helpOpen}>
          <Collapsible.Trigger
            class="flex w-full items-center justify-between text-sm font-medium"
          >
            <span>模板说明</span>
            <ChevronDown
              class="h-4 w-4 transition-transform {helpOpen
                ? 'rotate-180'
                : ''}"
            />
          </Collapsible.Trigger>
          <Collapsible.Content>
            <div
              class="mt-2 max-h-[400px] space-y-2 overflow-y-auto rounded-md border p-3"
            >
              <div class="space-y-1">
                <p class="text-muted-foreground">
                  <span class="text-foreground font-mono text-xs"
                    >{'{{alias}}'}</span
                  >
                  — 引用整个查询结果（集合类为数组，非集合类为对象）
                </p>
                <p class="text-muted-foreground">
                  <span class="text-foreground font-mono text-xs"
                    >{'{{alias.field}}'}</span
                  >
                  — 取单个字段值
                </p>
                <p class="text-muted-foreground">
                  <span class="text-foreground font-mono text-xs"
                    >{'{{alias_total}}'}</span
                  >
                  — 总记录数
                </p>
                <p class="text-muted-foreground">
                  <span class="text-foreground font-mono text-xs"
                    >{'{{alias_current}}'}</span
                  >
                  — 当前页码（从1开始）
                </p>
                <p class="text-muted-foreground">
                  <span class="text-foreground font-mono text-xs"
                    >{'{{alias_size}}'}</span
                  >
                  — 每页条数
                </p>
                <p class="text-muted-foreground">
                  <span class="text-foreground font-mono text-xs"
                    >{'{{alias_pages}}'}</span
                  >
                  — 总页数
                </p>
              </div>
              <div class="border-t pt-2">
                <p class="font-medium">非集合查询示例</p>
                <pre
                  class="bg-muted text-foreground mt-1 rounded p-1.5 font-mono text-[14px]">
{`{{user}} → {"id":1,"name":"张三"}
{{user.name}} → "张三"`}</pre>
              </div>
              <div class="border-t pt-2">
                <p class="font-medium">集合查询示例</p>
                <pre
                  class="bg-muted text-foreground mt-1 rounded p-1.5 font-mono text-[14px]">
{`{{orders}} → ["id":1, "id":2]
{{orders[0].id}} → 1
{{orders_total}} → 100`}</pre>
              </div>
              <div class="border-t pt-2">
                <p class="font-medium">同比/环比字段（非集合查询）</p>
                <p class="text-muted-foreground text-[11px]">
                  非集合查询且请求含 startDate/endDate 时，后端自动执行同比/环比
                  SQL
                </p>
                <div class="mt-1 space-y-1">
                  <p class="text-muted-foreground">
                    <span class="text-foreground font-mono text-xs"
                      >{'{{alias_yoy.field}}'}</span
                    >
                    — 同比期字段值
                  </p>
                  <p class="text-muted-foreground">
                    <span class="text-foreground font-mono text-xs"
                      >{'{{alias_mom.field}}'}</span
                    >
                    — 环比期字段值
                  </p>
                </div>
              </div>
              <div class="border-t pt-2">
                <p class="font-medium">比较函数</p>
                <div class="mt-1 space-y-1">
                  <p class="text-muted-foreground">
                    <span class="text-foreground font-mono text-xs"
                      >{'{{changeRate a b}}'}</span
                    >
                    — 变化率 (a-b)/b，如 0.25
                  </p>
                  <p class="text-muted-foreground">
                    <span class="text-foreground font-mono text-xs"
                      >{'{{changeRatePct a b}}'}</span
                    >
                    — 变化率百分数，如 25
                  </p>
                  <p class="text-muted-foreground">
                    <span class="text-foreground font-mono text-xs"
                      >{'{{delta a b}}'}</span
                    >
                    — 差值 a-b
                  </p>
                  <p class="text-muted-foreground">
                    <span class="text-foreground font-mono text-xs"
                      >{'{{fmtPct ratio decimals}}'}</span
                    >
                    — 格式化百分比，如 "10.0%"
                  </p>
                  <p class="text-muted-foreground">
                    <span class="text-foreground font-mono text-xs"
                      >{'{{default val fallback}}'}</span
                    >
                    — val 为 null 时返回 fallback
                  </p>
                </div>
              </div>
              <div class="border-t pt-2">
                <p class="font-medium">分页示例</p>
                <pre
                  class="bg-muted text-foreground mt-1 rounded p-1.5 font-mono text-[14px]">
{`{
  "list": {{orders}},
  "page": {
    "total": {{orders_total}},
    "current": {{orders_current}},
    "size": {{orders_size}},
    "pages": {{orders_pages}}
  }
}`}</pre>
              </div>
              <div class="border-t pt-2">
                <p class="font-medium">同比/环比完整示例</p>
                <pre
                  class="bg-muted text-foreground mt-1 rounded p-1.5 font-mono text-[14px]">
{`{
  "revenue": {{revenue.amt}},
  "revenue_yoy": {{revenue_yoy.amt}},
  "revenue_mom": {{revenue_mom.amt}},
  "yoy_rate": {{changeRate revenue.amt revenue_yoy.amt}},
  "yoy_pct": {{changeRatePct revenue.amt revenue_yoy.amt}},
  "yoy_text": {{fmtPct (changeRate revenue.amt revenue_yoy.amt) 1}}
}`}
                </pre>
              </div>
            </div>
          </Collapsible.Content>
        </Collapsible.Root>
        <Field.Error
          errors={errors.responseTemplate?.map((m: string) => ({
            message: m,
          })) ?? []}
        />
      </div>

      {#if errors._form}
        <div class="text-destructive text-sm">
          {errors._form.join(', ')}
        </div>
      {/if}

      {#if isLoadingDetail}
        <div class="text-muted-foreground text-sm">加载API详情...</div>
      {/if}
    </form>

    <Sheet.Footer class="flex flex-row justify-end gap-2">
      <Sheet.Close>
        <Button variant="outline" disabled={isSubmitting || isLoadingDetail}>
          取消
        </Button>
      </Sheet.Close>
      <Button
        form="httpapi-form"
        type="submit"
        disabled={isSubmitting || isLoadingDetail}
      >
        {#if isSubmitting || isLoadingDetail}
          <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
        {/if}
        {isEdit ? '保存' : isCopy ? '复制' : '创建'}
      </Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>
