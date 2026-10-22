<script lang="ts">
  import { untrack } from 'svelte'
  import { isApiClientNotifiedError } from '$lib/api/client.js'
  import { sqlSnippetsApi } from '$lib/api/sqlsnippets.js'
  import type {
    SqlSnippetCreateRequest,
    SqlSnippetListVo,
    SqlSnippetUpdateRequest,
  } from '$lib/api/types.js'
  import { Button } from '$lib/components/ui/button/index.js'
  import * as Field from '$lib/components/ui/field/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import * as Sheet from '$lib/components/ui/sheet/index.js'
  import { LoaderCircle } from '@lucide/svelte'
  import { toast } from 'svelte-sonner'
  import { z } from 'zod'

  interface Props {
    open: boolean
    mode: 'create' | 'update' | 'copy'
    initialSnippet: SqlSnippetListVo | null
    onOpenChange: (open: boolean) => void
    onsuccess?: () => void | Promise<void>
  }

  let { open, mode, initialSnippet, onOpenChange, onsuccess }: Props = $props()

  const formSchema = z.object({
    sqlSnippetName: z.string().min(1, '名称为必填项。'),
    sqlSnippetDesc: z.string().optional(),
    sqlSnippetContent: z.string().min(1, 'SQL 内容为必填项。'),
    tags: z.string().optional(),
  })

  type SnippetForm = z.infer<typeof formSchema>

  let formData = $state<SnippetForm>({
    sqlSnippetName: '',
    sqlSnippetDesc: '',
    sqlSnippetContent: '',
    tags: '',
  })

  let errors = $state<Record<string, string[]>>({})
  let isSubmitting = $state(false)

  const isUpdate = $derived(mode === 'update')
  const isCopy = $derived(mode === 'copy')

  function resetForm() {
    formData = {
      sqlSnippetName: '',
      sqlSnippetDesc: '',
      sqlSnippetContent: '',
      tags: '',
    }
    errors = {}
  }

  function syncFormFromProps() {
    if (mode === 'create') {
      resetForm()
      return
    }
    if (initialSnippet) {
      formData = {
        sqlSnippetName: initialSnippet.sqlSnippetName,
        sqlSnippetDesc: initialSnippet.sqlSnippetDesc || '',
        sqlSnippetContent: initialSnippet.sqlSnippetContent,
        tags: initialSnippet.tags || '',
      }
    }
    errors = {}
  }

  $effect(() => {
    if (open) {
      untrack(() => syncFormFromProps())
    }
  })

  function handleOpenChange(next: boolean) {
    onOpenChange(next)
    if (!next) {
      setTimeout(resetForm, 500)
    }
  }

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault()

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
      if (isUpdate && initialSnippet) {
        const input: SqlSnippetUpdateRequest = {
          sqlSnippetId: initialSnippet.sqlSnippetId,
          sqlSnippetName: parseResult.data.sqlSnippetName,
          sqlSnippetDesc: parseResult.data.sqlSnippetDesc || undefined,
          sqlSnippetContent: parseResult.data.sqlSnippetContent,
          tags: parseResult.data.tags || undefined,
        }
        await sqlSnippetsApi.update(input)
        toast.success(`片段 "${parseResult.data.sqlSnippetName}" 更新成功`)
      } else {
        const input: SqlSnippetCreateRequest = {
          sqlSnippetName: parseResult.data.sqlSnippetName,
          sqlSnippetDesc: parseResult.data.sqlSnippetDesc || undefined,
          sqlSnippetContent: parseResult.data.sqlSnippetContent,
          tags: parseResult.data.tags || undefined,
        }
        await sqlSnippetsApi.create(input)
        toast.success(
          `片段 "${parseResult.data.sqlSnippetName}" ${isCopy ? '复制' : '创建'}成功`
        )
      }
      onOpenChange(false)
      resetForm()
      await onsuccess?.()
    } catch (e) {
      const message =
        e instanceof Error
          ? e.message
          : isUpdate
            ? '更新片段失败'
            : '创建片段失败'
      if (!isApiClientNotifiedError(e)) {
        errors = { _form: [message] }
        toast.error(message)
      }
    } finally {
      isSubmitting = false
    }
  }
</script>

<Sheet.Root {open} onOpenChange={handleOpenChange}>
  <Sheet.Content class="flex flex-col">
    <Sheet.Header class="text-start">
      <Sheet.Title>
        {isUpdate ? '编辑片段' : isCopy ? '复制片段' : '创建片段'}
      </Sheet.Title>
      <Sheet.Description>
        {isUpdate
          ? '在这里输入一些SQL，以便您以后可以重用它。'
          : isCopy
            ? '基于现有片段创建副本，修改后保存。'
            : '添加新的 SQL 片段到库中，以便您以后可以重用它。'}
      </Sheet.Description>
    </Sheet.Header>

    <form
      id="snippet-form"
      onsubmit={handleSubmit}
      class="flex-1 space-y-4 overflow-y-auto px-4 py-4"
    >
      <Field.Field>
        <Field.Label>名称</Field.Label>
        <Input
          bind:value={formData.sqlSnippetName}
          placeholder="输入片段名称"
          disabled={isSubmitting}
        />
        <Field.Error
          errors={errors.sqlSnippetName?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>描述</Field.Label>
        <Input
          bind:value={formData.sqlSnippetDesc}
          placeholder="输入描述"
          disabled={isSubmitting}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>SQL 内容</Field.Label>
        <textarea
          bind:value={formData.sqlSnippetContent}
          placeholder="输入 SQL 内容"
          rows="6"
          class="border-input bg-background ring-offset-background placeholder:text-muted-foreground focus-visible:ring-ring flex min-h-[120px] w-full rounded-md border px-3 py-2 text-sm focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
          disabled={isSubmitting}
        ></textarea>
        <Field.Error
          errors={errors.sqlSnippetContent?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>标签</Field.Label>
        <Input
          bind:value={formData.tags}
          placeholder="输入标签（逗号分隔）"
          disabled={isSubmitting}
        />
      </Field.Field>

      {#if errors._form}
        <div class="text-destructive text-sm">
          {errors._form.join(', ')}
        </div>
      {/if}
    </form>

    <Sheet.Footer class="flex flex-row justify-end gap-2">
      <Sheet.Close>
        <Button variant="outline" disabled={isSubmitting}>取消</Button>
      </Sheet.Close>
      <Button form="snippet-form" type="submit" disabled={isSubmitting}>
        {#if isSubmitting}
          <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
        {/if}
        {isUpdate ? '保存' : isCopy ? '复制' : '创建'}
      </Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>
