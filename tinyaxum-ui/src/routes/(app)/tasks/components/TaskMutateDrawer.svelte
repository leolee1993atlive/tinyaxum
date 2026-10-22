<script lang="ts">
  import { isApiClientNotifiedError } from '$lib/api/client.js'
  import { transtaskApi } from '$lib/api/transtask.js'
  import { Button } from '$lib/components/ui/button/index.js'
  import * as Field from '$lib/components/ui/field/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import * as Sheet from '$lib/components/ui/sheet/index.js'
  import { LoaderCircle } from '@lucide/svelte'
  import { toast } from 'svelte-sonner'
  import { z } from 'zod'
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

  const formSchema = z.object({
    dataSourceId: z.number().min(1, '数据源为必填项。'),
    tableName: z.string().min(1, '表名为必填项。'),
    tableComment: z.string().min(1, '表注释为必填项。'),
    remark: z.string().optional(),
    taskCron: z.string().min(1, 'Cron表达式为必填项。'),
    syncMode: z.string().optional(),
    syncStrategy: z.string().optional(),
    syncColumn: z.string().optional(),
    syncPrimaryKey: z.string().optional(),
  })

  type TaskForm = z.infer<typeof formSchema>

  const defaultFormData = (): TaskForm => ({
    dataSourceId: 0,
    tableName: '',
    tableComment: '',
    remark: '',
    taskCron: '',
    syncMode: 'full',
    syncStrategy: '',
    syncColumn: '',
    syncPrimaryKey: '',
  })

  let formData = $state<TaskForm>(defaultFormData())
  const datasources = $derived(tasks.datasources)
  let errors = $state<Record<string, string[]>>({})
  let isSubmitting = $state(false)

  const isUpdate = $derived(tasks.open === 'update')
  const isCopy = $derived(tasks.open === 'copy')
  const needsIncrementalFields = $derived(
    formData.syncMode === 'incremental' || formData.syncMode === 'realtime'
  )
  const needsPrimaryKey = $derived(formData.syncStrategy === 'timestamp')

  // datasources 列表加载完后填默认值（仅在新建场景下且未选择数据源时）
  $effect(() => {
    if (!tasks.currentRow && datasources[0] && !formData.dataSourceId) {
      formData = {
        ...formData,
        dataSourceId: datasources[0].dataSourceId,
      }
    }
  })

  $effect(() => {
    if (
      (tasks.open === 'update' || tasks.open === 'copy') &&
      tasks.currentRow
    ) {
      formData = {
        dataSourceId: tasks.currentRow.dataSourceId,
        tableName: tasks.currentRow.tableName,
        tableComment: tasks.currentRow.tableComment,
        remark: tasks.currentRow.remark ?? '',
        taskCron: tasks.currentRow.taskCron,
        syncMode: tasks.currentRow.syncMode ?? 'full',
        syncStrategy: tasks.currentRow.syncStrategy ?? '',
        syncColumn: tasks.currentRow.syncColumn ?? '',
        syncPrimaryKey: tasks.currentRow.syncPrimaryKey ?? '',
      }
    } else if (tasks.open === 'create') {
      formData = {
        ...defaultFormData(),
        dataSourceId: datasources[0]?.dataSourceId ?? 0,
      }
    }
    errors = {}
  })

  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault()

    const parseResult = formSchema.safeParse(formData)
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
      if (isUpdate && tasks.currentRow) {
        await transtaskApi.update({
          transTaskId: tasks.currentRow.transTaskId,
          ...parseResult.data,
          taskStatus: tasks.currentRow.taskStatus,
        })
        toast.success('任务更新成功')
      } else {
        await transtaskApi.create({ ...parseResult.data, taskStatus: 'stop' })
        toast.success(isCopy ? '任务复制成功' : '任务创建成功')
      }

      tasks.setOpen(null)
      tasks.setCurrentRow(null)
      tasks.requestListRefetch()
    } catch (error) {
      const message = error instanceof Error ? error.message : '保存任务失败'
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
</script>

<Sheet.Root
  open={tasks.open === 'create' ||
    tasks.open === 'update' ||
    tasks.open === 'copy'}
  onOpenChange={handleOpenChange}
>
  <Sheet.Content class="flex flex-col">
    <Sheet.Header class="text-start">
      <Sheet.Title>
        {isUpdate ? '更新' : isCopy ? '复制' : '创建'}任务
      </Sheet.Title>
      <Sheet.Description>配置传输任务字段，完成后保存。</Sheet.Description>
    </Sheet.Header>

    <form
      id="tasks-form"
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
        <Field.Label>表名</Field.Label>
        <Input
          bind:value={formData.tableName}
          placeholder="例如：user_orders"
          disabled={isSubmitting}
        />
        <Field.Error
          errors={errors.tableName?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>表注释</Field.Label>
        <Input
          bind:value={formData.tableComment}
          placeholder="可读的任务描述"
          disabled={isSubmitting}
        />
        <Field.Error
          errors={errors.tableComment?.map((m) => ({ message: m })) ?? []}
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
          <Field.Description>源表中用于判断数据变化的列</Field.Description>
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
          placeholder="可选备注"
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
      <Button form="tasks-form" type="submit" disabled={isSubmitting}>
        {#if isSubmitting}
          <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
        {/if}
        保存
      </Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>
