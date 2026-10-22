<script lang="ts">
  import { untrack } from 'svelte'
  import { sqlSnippetsApi } from '$lib/api/sqlsnippets.js'
  import type { SqlSnippetListVo } from '$lib/api/types.js'
  import SqlSnippetFormDialog from '$lib/components/sql-snippet/sql-snippet-form-drawer.svelte'
  import * as Accordion from '$lib/components/ui/accordion/index.js'
  import { Badge } from '$lib/components/ui/badge/index.js'
  import { Button } from '$lib/components/ui/button/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js'
  import { cn } from '$lib/utils.js'
  import { createSnippetReference } from '$lib/utils/sql-params.js'
  import { ChevronDown, Code2, Pencil, Plus, Search } from '@lucide/svelte'
  import { Accordion as AccordionPrimitive } from 'bits-ui'

  interface Props {
    oninsert: (ref: string) => void
    /** 当前 SQL 中引用的片段名，这些片段置顶展示并打标 */
    usedNames?: string[]
  }

  let { oninsert, usedNames = [] }: Props = $props()

  let snippets: SqlSnippetListVo[] = $state([])
  let searchQuery = $state('')
  let loading = $state(true)
  let openItem = $state('')
  let snippetFormOpen = $state(false)
  let snippetFormMode = $state<'create' | 'update'>('create')
  let snippetFormSnippet = $state<SqlSnippetListVo | null>(null)

  $effect(() => {
    untrack(() => {
      void loadSnippets()
    })
  })

  async function loadSnippets() {
    loading = true
    try {
      const response = await sqlSnippetsApi.page({ pageNo: 1, pageSize: 200 })
      snippets = response.data?.records ?? []
    } catch {
      snippets = []
    } finally {
      loading = false
    }
  }

  /** 供父组件在片段被外部（如 AI 助手）修改后刷新列表 */
  export function reload() {
    void loadSnippets()
  }

  let filteredSnippets = $derived.by(() => {
    const list = snippets.filter((s) =>
      s.sqlSnippetName.toLowerCase().includes(searchQuery.toLowerCase())
    )
    // SQL 中引用的片段按引用出现顺序置顶，其余保持接口顺序（JS sort 稳定）
    const rank = new Map(usedNames.map((n, i) => [n, i]))
    return [...list].sort((a, b) => {
      const sa = rank.get(a.sqlSnippetName)
      const sb = rank.get(b.sqlSnippetName)
      if (sa !== undefined && sb !== undefined) return sa - sb
      if (sa !== undefined) return -1
      if (sb !== undefined) return 1
      return 0
    })
  })

  function isUsed(snippet: SqlSnippetListVo) {
    return usedNames.includes(snippet.sqlSnippetName)
  }

  function handleInsert(snippet: SqlSnippetListVo) {
    oninsert(createSnippetReference(snippet.sqlSnippetName))
  }

  function openCreateSnippet() {
    snippetFormSnippet = null
    snippetFormMode = 'create'
    snippetFormOpen = true
  }

  function openEditSnippet(snippet: SqlSnippetListVo) {
    snippetFormSnippet = snippet
    snippetFormMode = 'update'
    snippetFormOpen = true
  }

  function handleSnippetFormOpenChange(open: boolean) {
    snippetFormOpen = open
    if (!open) {
      setTimeout(() => {
        snippetFormSnippet = null
      }, 200)
    }
  }
</script>

<div
  data-slot="snippet-sidebar"
  class="bg-background flex h-full flex-col border-l"
>
  <div class="flex items-center justify-between gap-2 border-b px-3 py-2.5">
    <div class="flex min-w-0 items-center gap-2">
      <Code2 class="text-muted-foreground size-4 shrink-0" />
      <span class="truncate text-sm font-medium">片段</span>
    </div>
    <Button
      variant="outline"
      size="sm"
      class="h-8 shrink-0 gap-1 px-2 text-xs"
      onclick={openCreateSnippet}
    >
      <Plus class="size-3.5" />
      新增
    </Button>
  </div>

  <div class="border-b px-3 py-2">
    <div class="relative">
      <Search
        class="text-muted-foreground absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2"
      />
      <Input
        type="text"
        placeholder="搜索片段..."
        bind:value={searchQuery}
        class="h-8 pl-8 text-xs"
      />
    </div>
  </div>

  <ScrollArea class="min-h-0 flex-1">
    {#if loading}
      <div class="flex items-center justify-center py-8">
        <span class="text-muted-foreground text-xs">加载...</span>
      </div>
    {:else if filteredSnippets.length === 0}
      <div class="flex flex-col items-center justify-center gap-1.5 px-4 py-8">
        <Code2 class="text-muted-foreground/50 size-8" />
        <span class="text-muted-foreground text-xs">
          {#if searchQuery}
            没有片段匹配 "{searchQuery}"
          {:else}
            暂无
          {/if}
        </span>
      </div>
    {:else}
      <Accordion.Root
        type="single"
        class="w-full px-2 py-1"
        bind:value={openItem}
      >
        {#each filteredSnippets as snippet (snippet.sqlSnippetId)}
          <Accordion.Item
            value={String(snippet.sqlSnippetId)}
            class="border-b-0"
          >
            <AccordionPrimitive.Header
              class="flex w-full min-w-0 items-stretch gap-0 rounded-md"
              level={3}
            >
              <button
                type="button"
                class="hover:bg-accent flex max-w-[60%] min-w-0 shrink items-center gap-1.5 truncate rounded-md px-2 py-2 text-left text-sm leading-tight font-medium transition-colors"
                title={snippet.sqlSnippetName}
                onclick={() => handleInsert(snippet)}
              >
                {#if isUsed(snippet)}
                  <span
                    class="size-1.5 shrink-0 rounded-full bg-emerald-500"
                    aria-hidden="true"
                  ></span>
                {/if}
                <span class="truncate">{snippet.sqlSnippetName}</span>
              </button>
              <AccordionPrimitive.Trigger
                class={cn(
                  'group/trigger text-muted-foreground hover:bg-accent inline-flex min-w-8 flex-1 items-center justify-end rounded-md px-2 py-2 transition-colors',
                  'focus-visible:ring-ring/50 border border-transparent outline-none focus-visible:ring-2 focus-visible:ring-offset-2'
                )}
              >
                <ChevronDown
                  class="size-4 shrink-0 transition-transform duration-200 group-data-[state=open]/trigger:rotate-180"
                />
              </AccordionPrimitive.Trigger>
            </AccordionPrimitive.Header>
            <Accordion.Content class="px-2 pt-0 pb-2">
              <div
                class="bg-muted/30 space-y-2 rounded-md border p-2"
                onclick={(e) => e.stopPropagation()}
                role="presentation"
              >
                <div class="flex items-center justify-between gap-2">
                  <p class="text-muted-foreground text-xs leading-relaxed">
                    {#if snippet.sqlSnippetDesc?.trim()}
                      {snippet.sqlSnippetDesc}
                    {:else}
                      无描述
                    {/if}
                  </p>
                  {#if isUsed(snippet)}
                    <Badge
                      variant="secondary"
                      class="shrink-0 bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300"
                    >
                      使用中
                    </Badge>
                  {/if}
                </div>
                <pre
                  class="bg-background max-h-40 overflow-x-auto overflow-y-auto rounded border p-2 font-mono text-xs break-all whitespace-pre-wrap">{snippet.sqlSnippetContent}</pre>
                <Button
                  variant="secondary"
                  size="sm"
                  class="h-8 w-full gap-1 text-xs"
                  onclick={() => openEditSnippet(snippet)}
                >
                  <Pencil class="size-3.5" />
                  编辑
                </Button>
              </div>
            </Accordion.Content>
          </Accordion.Item>
        {/each}
      </Accordion.Root>
    {/if}
  </ScrollArea>
</div>

<SqlSnippetFormDialog
  open={snippetFormOpen}
  mode={snippetFormMode}
  initialSnippet={snippetFormMode === 'update' ? snippetFormSnippet : null}
  onOpenChange={handleSnippetFormOpenChange}
  onsuccess={loadSnippets}
/>
