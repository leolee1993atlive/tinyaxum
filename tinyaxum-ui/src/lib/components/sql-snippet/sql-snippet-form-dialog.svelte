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
  import * as Dialog from '$lib/components/ui/dialog/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import { Label } from '$lib/components/ui/label/index.js'
  import { LoaderCircle } from '@lucide/svelte'
  import { toast } from 'svelte-sonner'

  interface Props {
    open: boolean
    mode: 'create' | 'update'
    initialSnippet: SqlSnippetListVo | null
    onOpenChange: (open: boolean) => void
    onsuccess?: () => void | Promise<void>
  }

  let { open, mode, initialSnippet, onOpenChange, onsuccess }: Props = $props()

  const uid = $props.id()

  let isSubmitting = $state(false)
  let formData = $state({
    sqlSnippetName: '',
    sqlSnippetDesc: '',
    sqlSnippetContent: '',
    tags: '',
  })

  function resetForm() {
    formData = {
      sqlSnippetName: '',
      sqlSnippetDesc: '',
      sqlSnippetContent: '',
      tags: '',
    }
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
  }

  $effect(() => {
    if (open) {
      untrack(() => syncFormFromProps())
    }
  })

  function handleClose(next: boolean) {
    onOpenChange(next)
    if (!next) {
      setTimeout(resetForm, 200)
    }
  }

  async function handleCreate() {
    if (!formData.sqlSnippetName || !formData.sqlSnippetContent) {
      toast.error('名称和内容为必填项')
      return
    }

    isSubmitting = true
    try {
      const input: SqlSnippetCreateRequest = {
        sqlSnippetName: formData.sqlSnippetName,
        sqlSnippetDesc: formData.sqlSnippetDesc || undefined,
        sqlSnippetContent: formData.sqlSnippetContent,
        tags: formData.tags || undefined,
      }
      await sqlSnippetsApi.create(input)
      toast.success(`片段 "${formData.sqlSnippetName}" 创建成功`)
      onOpenChange(false)
      resetForm()
      await onsuccess?.()
    } catch (e) {
      const message = e instanceof Error ? e.message : '创建片段失败'
      if (!isApiClientNotifiedError(e)) {
        toast.error(message)
      }
    } finally {
      isSubmitting = false
    }
  }

  async function handleUpdate() {
    if (
      !initialSnippet ||
      !formData.sqlSnippetName ||
      !formData.sqlSnippetContent
    ) {
      toast.error('名称和内容为必填项')
      return
    }

    isSubmitting = true
    try {
      const input: SqlSnippetUpdateRequest = {
        sqlSnippetId: initialSnippet.sqlSnippetId,
        sqlSnippetName: formData.sqlSnippetName,
        sqlSnippetDesc: formData.sqlSnippetDesc || undefined,
        sqlSnippetContent: formData.sqlSnippetContent,
        tags: formData.tags || undefined,
      }
      await sqlSnippetsApi.update(input)
      toast.success(`片段 "${formData.sqlSnippetName}" 更新成功`)
      onOpenChange(false)
      resetForm()
      await onsuccess?.()
    } catch (e) {
      const message = e instanceof Error ? e.message : '更新片段失败'
      if (!isApiClientNotifiedError(e)) {
        toast.error(message)
      }
    } finally {
      isSubmitting = false
    }
  }
</script>

<Dialog.Root {open} onOpenChange={handleClose}>
  <Dialog.Content class="sm:max-w-lg">
    <Dialog.Header>
      <Dialog.Title>
        {mode === 'create' ? '创建片段' : '编辑片段'}
      </Dialog.Title>
      <Dialog.Description>
        {mode === 'create'
          ? '添加新的 SQL 片段到库中，以便您以后可以重用它。'
          : '在这里输入一些SQL，以便您以后可以重用它'}
      </Dialog.Description>
    </Dialog.Header>

    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="sql-snippet-name-{uid}">名称</Label>
        <Input
          id="sql-snippet-name-{uid}"
          bind:value={formData.sqlSnippetName}
          placeholder="输入片段名称"
          disabled={isSubmitting}
        />
      </div>

      <div class="grid gap-2">
        <Label for="sql-snippet-desc-{uid}">描述</Label>
        <Input
          id="sql-snippet-desc-{uid}"
          bind:value={formData.sqlSnippetDesc}
          placeholder="输入描述"
          disabled={isSubmitting}
        />
      </div>

      <div class="grid gap-2">
        <Label for="sql-snippet-content-{uid}">SQL 内容</Label>
        <textarea
          id="sql-snippet-content-{uid}"
          bind:value={formData.sqlSnippetContent}
          placeholder="输入 SQL 内容"
          rows="6"
          class="border-input bg-background ring-offset-background placeholder:text-muted-foreground focus-visible:ring-ring flex min-h-[120px] w-full rounded-md border px-3 py-2 text-sm focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
          disabled={isSubmitting}
        ></textarea>
      </div>

      <div class="grid gap-2">
        <Label for="sql-snippet-tags-{uid}">标签</Label>
        <Input
          id="sql-snippet-tags-{uid}"
          bind:value={formData.tags}
          placeholder="输入标签（逗号分隔）"
          disabled={isSubmitting}
        />
      </div>
    </div>

    <Dialog.Footer>
      <Button
        variant="outline"
        onclick={() => handleClose(false)}
        disabled={isSubmitting}
      >
        取消
      </Button>
      <Button
        onclick={mode === 'create' ? handleCreate : handleUpdate}
        disabled={isSubmitting}
      >
        {#if isSubmitting}
          <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
        {/if}
        {mode === 'create' ? '创建' : '保存'}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
