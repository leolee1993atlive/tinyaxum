<script lang="ts">
  import type { DatasourceListVo } from '$lib/api/types.js'
  import { Button } from '$lib/components/ui/button/index.js'
  import * as Select from '$lib/components/ui/select/index.js'
  import { X } from '@lucide/svelte'

  type Props = {
    value: number | null
    datasources: DatasourceListVo[]
    placeholder?: string
    onChange?: (id: number | null) => void
    class?: string
  }

  let {
    value,
    datasources,
    placeholder = '全部数据源',
    onChange,
    class: className,
  }: Props = $props()

  const ALL_VALUE = '__all__'

  let stringValue = $derived(value !== null ? String(value) : ALL_VALUE)

  const selectedLabel = $derived.by(() => {
    if (value === null) return placeholder
    const ds = datasources.find((d) => d.dataSourceId === value)
    return ds ? `${ds.name} (${ds.dbType})` : placeholder
  })

  function handleValueChange(next: string) {
    if (next === ALL_VALUE) {
      onChange?.(null)
    } else {
      onChange?.(Number(next))
    }
  }

  function clear() {
    onChange?.(null)
  }
</script>

<div class="flex items-center gap-1" data-slot="datasource-filter">
  <Select.Root
    type="single"
    value={stringValue}
    onValueChange={handleValueChange}
  >
    <Select.Trigger class={className ?? 'w-48'}>
      <span class="truncate">{selectedLabel}</span>
    </Select.Trigger>
    <Select.Content>
      <Select.Item value={ALL_VALUE}>{placeholder}</Select.Item>
      {#if datasources.length === 0}
        <Select.Item value="__empty__" disabled>无可用数据源</Select.Item>
      {:else}
        <Select.Separator />
        <Select.Group>
          {#each datasources as ds (ds.dataSourceId)}
            <Select.Item value={String(ds.dataSourceId)} label={ds.name}>
              {ds.name} ({ds.dbType})
            </Select.Item>
          {/each}
        </Select.Group>
      {/if}
    </Select.Content>
  </Select.Root>
</div>
