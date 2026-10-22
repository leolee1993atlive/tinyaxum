<script lang="ts">
  import { untrack } from 'svelte'
  import { sqlQueriesApi } from '$lib/api/sqlqueries.js'
  import type { SqlQueryListVo } from '$lib/api/types.js'
  import * as Select from '$lib/components/ui/select/index.js'
  import { FileText, Plus } from '@lucide/svelte'

  interface Props {
    value?: number | null
    onselect?: (query: SqlQueryListVo | null) => void
  }

  let { value = $bindable<number | null>(null), onselect }: Props = $props()

  let queries: SqlQueryListVo[] = $state([])
  let loading = $state(true)

  const NEW_QUERY_VALUE = '__new__'

  let stringValue = $derived(value !== null ? String(value) : NEW_QUERY_VALUE)

  const selectedLabel = $derived(
    value !== null
      ? (queries.find((q) => q.sqlQueryId === value)?.sqlQueryName ??
          '选择查询...')
      : '新建查询'
  )

  $effect(() => {
    untrack(() => {
      void loadQueries()
    })
  })

  async function loadQueries() {
    loading = true
    try {
      // 下拉选项：首屏拉取最多 200 条；完整列表需服务端搜索或分页下拉。
      const response = await sqlQueriesApi.list()
      queries = response.data ?? []
    } catch {
      queries = []
    } finally {
      loading = false
    }
  }

  function handleValueChange(newValue: string) {
    if (newValue === NEW_QUERY_VALUE) {
      value = null
      onselect?.(null)
    } else {
      const id = Number(newValue)
      value = id
      const query = queries.find((q) => q.sqlQueryId === id)
      if (query) {
        onselect?.(query)
      }
    }
  }
</script>

<div data-slot="query-selector">
  <Select.Root
    type="single"
    value={stringValue}
    onValueChange={handleValueChange}
  >
    <Select.Trigger class="w-full">
      <div class="flex items-center gap-2">
        <FileText class="size-4 shrink-0" />
        <span>{selectedLabel}</span>
      </div>
    </Select.Trigger>
    <Select.Content>
      <Select.Item value={NEW_QUERY_VALUE}>
        <div class="flex items-center gap-2">
          <Plus class="size-4" />
          <span>新建查询</span>
        </div>
      </Select.Item>
      <Select.Separator />
      {#if loading}
        <Select.Item value="__loading__" disabled>
          Loading queries...
        </Select.Item>
      {:else if queries.length === 0}
        <Select.Item value="__empty__" disabled>No saved queries</Select.Item>
      {:else}
        <Select.Group>
          {#each queries as query (query.sqlQueryId)}
            <Select.Item
              value={String(query.sqlQueryId)}
              label={query.sqlQueryName}
            >
              {query.sqlQueryName}
            </Select.Item>
          {/each}
        </Select.Group>
      {/if}
    </Select.Content>
  </Select.Root>
</div>
