<script lang="ts">
  import { untrack } from 'svelte'
  import { isApiClientNotifiedError } from '$lib/api/client.js'
  import { sqlSnippetsApi } from '$lib/api/sqlsnippets.js'
  import type { SqlSnippetListVo } from '$lib/api/types.js'
  import SelectDropdown from '$lib/components/SelectDropdown.svelte'
  import { Badge } from '$lib/components/ui/badge/index.js'
  import { Button } from '$lib/components/ui/button/index.js'
  import * as Dialog from '$lib/components/ui/dialog/index.js'
  import { Label } from '$lib/components/ui/label/index.js'
  import { LoaderCircle } from '@lucide/svelte'
  import { toast } from 'svelte-sonner'

  interface Props {
    open: boolean
    /** AI 输出的代码块内容，将替换目标片段的 SQL 内容 */
    code: string
    /** 当前 SQL 中引用的片段名，用于置顶与预选 */
    usedSnippetNames?: string[]
    onOpenChange: (open: boolean) => void
    /** 保存成功后回调（父组件借此刷新片段映射/侧栏） */
    onApplied?: () => void | Promise<void>
  }

  let {
    open,
    code,
    usedSnippetNames = [],
    onOpenChange,
    onApplied,
  }: Props = $props()

  let snippets = $state<SqlSnippetListVo[]>([])
  let loading = $state(false)
  let selectedId = $state<number | null>(null)
  let applying = $state(false)

  const newContent = $derived(code.trim())

  // 当前 SQL 用到的片段排在最前（按引用出现顺序），其余保持接口顺序
  const orderedSnippets = $derived.by(() => {
    const rank = new Map(usedSnippetNames.map((n, i) => [n, i]))
    const score = (s: SqlSnippetListVo) => rank.get(s.sqlSnippetName)
    return [...snippets].sort((a, b) => {
      const sa = score(a)
      const sb = score(b)
      if (sa !== undefined && sb !== undefined) return sa - sb
      if (sa !== undefined) return -1
      if (sb !== undefined) return 1
      return 0
    })
  })

  const selected = $derived(
    snippets.find((s) => s.sqlSnippetId === selectedId) ?? null
  )

  const unchanged = $derived(selected?.sqlSnippetContent === newContent)

  $effect(() => {
    if (open) {
      untrack(() => {
        void loadSnippets()
      })
    }
  })

  async function loadSnippets() {
    loading = true
    selectedId = null
    try {
      const res = await sqlSnippetsApi.list()
      snippets = res.data ?? []
      // 预选：优先当前 SQL 用到的第一个片段
      const used = orderedSnippets.find((s) =>
        usedSnippetNames.includes(s.sqlSnippetName)
      )
      selectedId =
        used?.sqlSnippetId ?? orderedSnippets[0]?.sqlSnippetId ?? null
    } catch (e) {
      snippets = []
      if (!isApiClientNotifiedError(e)) {
        toast.error(e instanceof Error ? e.message : '加载片段列表失败')
      }
    } finally {
      loading = false
    }
  }

  async function handleApply() {
    if (!selected || unchanged || applying) return
    applying = true
    try {
      await sqlSnippetsApi.update({
        sqlSnippetId: selected.sqlSnippetId,
        sqlSnippetName: selected.sqlSnippetName,
        sqlSnippetDesc: selected.sqlSnippetDesc || undefined,
        sqlSnippetContent: newContent,
        tags: selected.tags || undefined,
      })
      toast.success(`片段 "${selected.sqlSnippetName}" 已更新`)
      onOpenChange(false)
      await onApplied?.()
    } catch (e) {
      if (!isApiClientNotifiedError(e)) {
        toast.error(e instanceof Error ? e.message : '更新片段失败')
      }
    } finally {
      applying = false
    }
  }
</script>

<Dialog.Root {open} {onOpenChange}>
  <Dialog.Content class="flex max-h-[80vh] flex-col sm:max-w-2xl">
    <Dialog.Header>
      <Dialog.Title>应用到片段</Dialog.Title>
      <Dialog.Description>
        将这段 SQL 覆盖写入所选片段（名称、描述、标签保持不变），无需关闭 AI
        助手。
      </Dialog.Description>
    </Dialog.Header>

    {#if loading}
      <div class="flex items-center justify-center py-8">
        <LoaderCircle class="text-muted-foreground h-6 w-6 animate-spin" />
      </div>
    {:else if orderedSnippets.length === 0}
      <p class="text-muted-foreground py-6 text-sm">暂无可选片段</p>
    {:else}
      <div class="flex flex-col gap-1.5">
        <Label class="text-muted-foreground text-xs">目标片段</Label>
        <SelectDropdown
          value={selectedId == null ? undefined : String(selectedId)}
          onValueChange={(v) => (selectedId = Number(v))}
          items={orderedSnippets.map((s) => ({
            label: usedSnippetNames.includes(s.sqlSnippetName)
              ? `${s.sqlSnippetName} · 使用中`
              : s.sqlSnippetName,
            value: String(s.sqlSnippetId),
          }))}
          placeholder="选择要覆盖的片段"
          disabled={applying}
        />
        {#if selected}
          <p class="text-muted-foreground text-xs">
            {selected.sqlSnippetDesc?.trim() || '无描述'}
          </p>
        {/if}
      </div>

      <div
        class="grid min-h-0 flex-1 grid-cols-1 gap-2 overflow-y-auto sm:grid-cols-2"
      >
        <div class="min-w-0 space-y-1">
          <p class="text-muted-foreground text-xs font-medium">当前片段内容</p>
          <pre
            class="bg-muted/50 max-h-56 overflow-auto rounded-md border p-2 font-mono text-xs break-all whitespace-pre-wrap">{selected?.sqlSnippetContent ??
              ''}</pre>
        </div>
        <div class="min-w-0 space-y-1">
          <p
            class="text-muted-foreground flex items-center gap-1.5 text-xs font-medium"
          >
            AI 代码（新内容）
            {#if unchanged}
              <Badge variant="secondary">内容一致</Badge>
            {/if}
          </p>
          <pre
            class="bg-muted/50 max-h-56 overflow-auto rounded-md border p-2 font-mono text-xs break-all whitespace-pre-wrap">{newContent}</pre>
        </div>
      </div>
    {/if}

    <Dialog.Footer>
      <Button
        variant="outline"
        onclick={() => onOpenChange(false)}
        disabled={applying}
      >
        取消
      </Button>
      <Button
        onclick={handleApply}
        disabled={loading || !selected || unchanged || applying}
      >
        {#if applying}
          <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
        {/if}
        保存到片段
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
