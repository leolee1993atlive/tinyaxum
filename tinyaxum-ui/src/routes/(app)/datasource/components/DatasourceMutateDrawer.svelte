<script lang="ts">
  import { isApiClientNotifiedError } from '$lib/api/client.js'
  import { datasourcesApi } from '$lib/api/datasources.js'
  import type { DatasourceDetailVo } from '$lib/api/types.js'
  import { Button } from '$lib/components/ui/button/index.js'
  import * as Field from '$lib/components/ui/field/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import * as Sheet from '$lib/components/ui/sheet/index.js'
  import { LoaderCircle } from '@lucide/svelte'
  import { toast } from 'svelte-sonner'
  import { z } from 'zod'
  import { useDatasources } from './datasource-state.svelte'

  const datasources = useDatasources()

  const formSchema = z.object({
    code: z.string().min(1, '编码为必填项。'),
    name: z.string().min(1, '名称为必填项。'),
    dbType: z.string().min(1, '数据库类型为必填项。'),
    dbHost: z.string().min(1, '主机为必填项。'),
    dbPort: z
      .number()
      .min(1, '端口为必填项。')
      .max(65535, '端口必须在 1 到 65535 之间。'),
    dbName: z.string().min(1, '数据库名称为必填项。'),
    dbSchema: z.string().max(128, 'Schema 最长 128 字符。').optional(),
    dbUsername: z.string().min(1, '用户名为必填项。'),
    dbPassword: z.string().optional(),
    remark: z.string().optional(),
  })

  type DatasourceForm = z.infer<typeof formSchema>

  let formData = $state<DatasourceForm>({
    code: '',
    name: '',
    dbType: 'mysql',
    dbHost: 'localhost',
    dbPort: 3306,
    dbName: '',
    dbSchema: 'public',
    dbUsername: '',
    dbPassword: '',
    remark: '',
  })

  let errors = $state<Record<string, string[]>>({})
  let isSubmitting = $state(false)
  let isLoadingDetail = $state(false)

  const isCreate = $derived(datasources.open === 'create')
  const isUpdate = $derived(datasources.open === 'update')
  const isCopy = $derived(datasources.open === 'copy')

  function mapDetailToForm(detail: DatasourceDetailVo): DatasourceForm {
    return {
      code: detail.code,
      name: detail.name,
      dbType: detail.dbType,
      dbHost: detail.dbHost,
      dbPort: detail.dbPort,
      dbName: detail.dbName,
      dbSchema: detail.dbSchema ?? 'public',
      dbUsername: detail.dbUsername,
      dbPassword: '',
      remark: detail.remark ?? '',
    }
  }

  async function loadDatasourceDetail(dataSourceId: number) {
    isLoadingDetail = true
    try {
      const response = await datasourcesApi.getById(dataSourceId)
      if (response.data) {
        formData = mapDetailToForm(response.data)
      }
    } catch (error) {
      const message =
        error instanceof Error ? error.message : '加载数据源详情失败'
      if (isApiClientNotifiedError(error)) {
        errors = {}
      } else {
        errors = { _form: [message] }
        toast.error(message)
      }
    } finally {
      isLoadingDetail = false
    }
  }

  $effect(() => {
    if (datasources.open === 'copy' && datasources.copyFormData) {
      formData = { ...datasources.copyFormData }
    } else if (datasources.open === 'update' && datasources.currentRow) {
      // 列表行只有元数据；主机/端口/用户名等详情接口回来后覆盖。
      // 端口先按类型给常见默认值，避免加载完成前出现误导性的假数据。
      formData = {
        code: datasources.currentRow.code,
        name: datasources.currentRow.name,
        dbType: datasources.currentRow.dbType,
        dbHost: '',
        dbPort: datasources.currentRow.dbType === 'postgresql' ? 5432 : 3306,
        dbName: datasources.currentRow.dbName,
        dbSchema: 'public',
        dbUsername: '',
        dbPassword: '',
        remark: datasources.currentRow.remark ?? '',
      }
      void loadDatasourceDetail(datasources.currentRow.dataSourceId)
    } else if (datasources.open === 'create') {
      formData = {
        code: '',
        name: '',
        dbType: 'mysql',
        dbHost: 'localhost',
        dbPort: 3306,
        dbName: '',
        dbSchema: 'public',
        dbUsername: '',
        dbPassword: '',
        remark: '',
      }
    }
    errors = {}
  })

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault()

    // For create mode, password is required
    const schemaToUse = isUpdate
      ? formSchema
      : formSchema.extend({
          dbPassword: z.string().min(1, '密码为必填项。'),
        })

    const parseResult = schemaToUse.safeParse(formData)

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
      if (isUpdate && datasources.currentRow) {
        const updateData = {
          dataSourceId: datasources.currentRow.dataSourceId,
          ...parseResult.data,
          dbPassword: parseResult.data.dbPassword || '',
          // 非 PostgreSQL 数据源不携带 schema，避免脏数据残留
          dbSchema:
            parseResult.data.dbType === 'postgresql'
              ? parseResult.data.dbSchema
              : undefined,
        }
        await datasourcesApi.update(updateData)
        toast.success('数据源更新成功')
      } else {
        await datasourcesApi.create({
          code: parseResult.data.code,
          name: parseResult.data.name,
          dbType: parseResult.data.dbType,
          dbHost: parseResult.data.dbHost,
          dbPort: parseResult.data.dbPort,
          dbName: parseResult.data.dbName,
          dbSchema:
            parseResult.data.dbType === 'postgresql'
              ? parseResult.data.dbSchema
              : undefined,
          dbUsername: parseResult.data.dbUsername,
          dbPassword: parseResult.data.dbPassword || '',
          remark: parseResult.data.remark,
        })
        toast.success(isCopy ? '数据源复制成功' : '数据源创建成功')
      }
      datasources.setOpen(null)
      datasources.requestListRefetch()
    } catch (e) {
      const message = e instanceof Error ? e.message : '保存数据源失败'
      if (isApiClientNotifiedError(e)) {
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
      datasources.setOpen(null)
      setTimeout(() => {
        datasources.setCurrentRow(null)
        datasources.setCopyFormData(null)
      }, 500)
    }
  }
</script>

<Sheet.Root
  open={datasources.open === 'create' ||
    datasources.open === 'update' ||
    datasources.open === 'copy'}
  onOpenChange={handleOpenChange}
>
  <Sheet.Content class="flex flex-col">
    <Sheet.Header class="text-start">
      <Sheet.Title
        >{isUpdate ? '更新' : isCopy ? '复制' : '创建'}数据源</Sheet.Title
      >
      <Sheet.Description>
        {isUpdate ? '更新数据源连接详情。' : '提供连接详情以添加新数据源。'}
        完成后点击保存。
      </Sheet.Description>
    </Sheet.Header>

    <form
      id="datasource-form"
      onsubmit={handleSubmit}
      class="flex-1 space-y-4 overflow-y-auto px-4 py-4"
    >
      <Field.Field>
        <Field.Label>编码</Field.Label>
        <Input
          bind:value={formData.code}
          placeholder="例如：PROD_DB"
          disabled={isSubmitting || isLoadingDetail}
        />
        <Field.Error errors={errors.code?.map((m) => ({ message: m })) ?? []} />
      </Field.Field>

      <Field.Field>
        <Field.Label>名称</Field.Label>
        <Input
          bind:value={formData.name}
          placeholder="例如：生产数据库"
          disabled={isSubmitting || isLoadingDetail}
        />
        <Field.Error errors={errors.name?.map((m) => ({ message: m })) ?? []} />
      </Field.Field>

      <Field.Field>
        <Field.Label>数据库类型</Field.Label>
        <select
          bind:value={formData.dbType}
          class="border-input bg-background ring-offset-background focus-visible:ring-ring flex h-10 w-full rounded-md border px-3 py-2 text-sm focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
          disabled={isSubmitting || isLoadingDetail}
        >
          <option value="mysql">MySQL</option>
          <option value="postgresql">PostgreSQL</option>
          <option value="duckdb">DuckDB</option>
        </select>
        <Field.Error
          errors={errors.dbType?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>主机</Field.Label>
        <Input
          bind:value={formData.dbHost}
          placeholder="例如：localhost 或 192.168.1.100"
          disabled={isSubmitting || isLoadingDetail}
        />
        <Field.Error
          errors={errors.dbHost?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>端口</Field.Label>
        <Input
          type="number"
          bind:value={formData.dbPort}
          placeholder="例如：3306"
          disabled={isSubmitting || isLoadingDetail}
        />
        <Field.Error
          errors={errors.dbPort?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>数据库名称</Field.Label>
        <Input
          bind:value={formData.dbName}
          placeholder="例如：mydb"
          disabled={isSubmitting || isLoadingDetail}
        />
        <Field.Error
          errors={errors.dbName?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      {#if formData.dbType === 'postgresql'}
        <Field.Field>
          <Field.Label>Schema</Field.Label>
          <Input
            bind:value={formData.dbSchema}
            placeholder="默认 public"
            disabled={isSubmitting || isLoadingDetail}
          />
          <Field.Description>
            PostgreSQL 模式名，留空默认 public
          </Field.Description>
          <Field.Error
            errors={errors.dbSchema?.map((m) => ({ message: m })) ?? []}
          />
        </Field.Field>
      {/if}

      <Field.Field>
        <Field.Label>用户名</Field.Label>
        <Input
          bind:value={formData.dbUsername}
          placeholder="例如：root"
          disabled={isSubmitting || isLoadingDetail}
        />
        <Field.Error
          errors={errors.dbUsername?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>密码{isUpdate ? '（留空保持当前密码）' : ''}</Field.Label>
        <Input
          type="password"
          bind:value={formData.dbPassword}
          placeholder={isUpdate ? '留空保持当前密码' : '输入密码'}
          disabled={isSubmitting || isLoadingDetail}
        />
        <Field.Error
          errors={errors.dbPassword?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>备注</Field.Label>
        <Input
          bind:value={formData.remark}
          placeholder="可选备注"
          disabled={isSubmitting || isLoadingDetail}
        />
        <Field.Error
          errors={errors.remark?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      {#if errors._form}
        <div class="text-destructive text-sm">
          {errors._form.join(', ')}
        </div>
      {/if}

      {#if isLoadingDetail}
        <div class="text-muted-foreground text-sm">加载数据源详情...</div>
      {/if}
    </form>

    <Sheet.Footer class="flex flex-row justify-end gap-2">
      <Sheet.Close>
        <Button variant="outline" disabled={isSubmitting || isLoadingDetail}
          >关闭</Button
        >
      </Sheet.Close>
      <Button
        form="datasource-form"
        type="submit"
        disabled={isSubmitting || isLoadingDetail}
      >
        {#if isSubmitting || isLoadingDetail}
          <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
        {/if}
        保存
      </Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>
