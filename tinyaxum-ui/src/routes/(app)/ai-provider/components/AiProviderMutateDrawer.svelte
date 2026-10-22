<script lang="ts">
  import { aiProvidersApi } from '$lib/api/ai-providers.js'
  import { isApiClientNotifiedError } from '$lib/api/client.js'
  import type { AiProviderListVo } from '$lib/api/types.js'
  import { Button } from '$lib/components/ui/button/index.js'
  import * as Field from '$lib/components/ui/field/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import * as Sheet from '$lib/components/ui/sheet/index.js'
  import { Textarea } from '$lib/components/ui/textarea/index.js'
  import { LoaderCircle } from '@lucide/svelte'
  import { toast } from 'svelte-sonner'
  import { z } from 'zod'
  import { useAiProviders } from './ai-provider-state.svelte'

  const aiProviderCtx = useAiProviders()

  const formSchema = z.object({
    name: z.string().min(1, '名称为必填项。'),
    baseUrl: z.string().min(1, 'Base URL 为必填项。'),
    apiKey: z.string(),
    model: z.string().min(1, '模型为必填项。'),
    systemPrompt: z.string().min(1, 'System Prompt 为必填项。'),
    temperature: z.number().min(0, '最小值为 0').max(2, '最大值为 2'),
  })

  type Form = z.infer<typeof formSchema>

  let formData = $state<Form>({
    name: '',
    baseUrl: '',
    apiKey: '',
    model: '',
    systemPrompt:
      '你是一名资深数据分析师,擅长将 MySQL SQL 改写为 DuckDB SQL,精简多余 select 字段,并保留语义等价。',
    temperature: 0.3,
  })

  let errors = $state<Record<string, string[]>>({})
  let isSubmitting = $state(false)

  const isUpdate = $derived(aiProviderCtx.open === 'update')
  const isCopy = $derived(aiProviderCtx.open === 'copy')

  function mapProviderToForm(p: AiProviderListVo): Form {
    return {
      name: p.name,
      baseUrl: p.baseUrl,
      apiKey: '',
      model: p.model,
      systemPrompt: p.systemPrompt,
      temperature: p.temperature,
    }
  }

  $effect(() => {
    if (
      (isUpdate || isCopy) &&
      (aiProviderCtx.currentRow || aiProviderCtx.copyProviderData)
    ) {
      const source = isCopy
        ? aiProviderCtx.copyProviderData!
        : aiProviderCtx.currentRow!
      formData = mapProviderToForm(source)
    } else if (aiProviderCtx.open === 'create') {
      formData = {
        name: '',
        baseUrl: '',
        apiKey: '',
        model: '',
        systemPrompt:
          '你是一名资深数据分析师,擅长将 MySQL SQL 改写为 DuckDB SQL,精简多余 select 字段,并保留语义等价。',
        temperature: 0.3,
      }
    }
    errors = {}
  })

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault()

    const schemaToUse = isUpdate
      ? formSchema
      : formSchema.extend({
          apiKey: z.string().min(1, 'API Key 为必填项。'),
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
      if (isUpdate && aiProviderCtx.currentRow) {
        await aiProvidersApi.update(
          aiProviderCtx.currentRow.aiProviderId,
          formData
        )
        toast.success('更新成功')
      } else {
        await aiProvidersApi.create(formData)
        toast.success(isCopy ? '模型复制成功' : '创建成功')
      }

      aiProviderCtx.closeDialogs()
      aiProviderCtx.requestListRefetch()
    } catch (error) {
      const message = error instanceof Error ? error.message : '保存失败'
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
      aiProviderCtx.closeDialogs()
      aiProviderCtx.resetCopyProvider()
    }
  }
</script>

<Sheet.Root
  open={aiProviderCtx.open === 'create' ||
    aiProviderCtx.open === 'update' ||
    aiProviderCtx.open === 'copy'}
  onOpenChange={handleOpenChange}
>
  <Sheet.Content class="flex flex-col">
    <Sheet.Header class="text-start">
      <Sheet.Title
        >{isUpdate ? '编辑模型' : isCopy ? '复制模型' : '新增模型'}</Sheet.Title
      >
      <Sheet.Description>
        {isUpdate
          ? '更新模型供应商配置。'
          : isCopy
            ? '基于现有模型创建副本，修改后保存。'
            : '提供 OpenAI 兼容的 LLM 供应商配置。'}
        完成后点击保存。
      </Sheet.Description>
    </Sheet.Header>

    <form
      id="ai-provider-form"
      onsubmit={handleSubmit}
      class="flex-1 space-y-4 overflow-y-auto px-4 py-4"
    >
      <Field.Field>
        <Field.Label>名称</Field.Label>
        <Input bind:value={formData.name} disabled={isSubmitting} />
        <Field.Error errors={errors.name?.map((m) => ({ message: m })) ?? []} />
      </Field.Field>

      <Field.Field>
        <Field.Label>Base URL</Field.Label>
        <Input
          bind:value={formData.baseUrl}
          placeholder="https://api.deepseek.com/v1"
          disabled={isSubmitting}
        />
        <Field.Error
          errors={errors.baseUrl?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>API Key</Field.Label>
        <Input
          type="password"
          bind:value={formData.apiKey}
          placeholder={isUpdate ? '留空保持不变' : 'sk-...'}
          disabled={isSubmitting}
        />
        <Field.Error
          errors={errors.apiKey?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>模型</Field.Label>
        <Input
          bind:value={formData.model}
          placeholder="deepseek-chat"
          disabled={isSubmitting}
        />
        <Field.Error
          errors={errors.model?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>Temperature</Field.Label>
        <Input
          type="number"
          step="0.1"
          min="0"
          max="2"
          bind:value={formData.temperature}
          disabled={isSubmitting}
        />
        <Field.Error
          errors={errors.temperature?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>System Prompt</Field.Label>
        <Textarea
          rows={6}
          bind:value={formData.systemPrompt}
          disabled={isSubmitting}
        />
        <Field.Error
          errors={errors.systemPrompt?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      {#if errors._form}
        <div class="text-destructive text-sm">{errors._form[0]}</div>
      {/if}
    </form>

    <Sheet.Footer class="flex flex-row justify-end gap-2">
      <Sheet.Close>
        <Button variant="outline" disabled={isSubmitting}>关闭</Button>
      </Sheet.Close>
      <Button form="ai-provider-form" type="submit" disabled={isSubmitting}>
        {#if isSubmitting}
          <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
        {/if}
        保存
      </Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>
