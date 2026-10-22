<script lang="ts">
  import type { ModuleOptionVo } from '$lib/api/types.js'
  import * as Select from '$lib/components/ui/select/index.js'

  let {
    moduleOptions = [],
    module = $bindable(null as string | null),
    category = $bindable(null as string | null),
    onChange,
  }: {
    moduleOptions: ModuleOptionVo[]
    module?: string | null
    category?: string | null
    onChange?: () => void
  } = $props()

  let categoryOptions = $derived.by(() => {
    if (!module) {
      const all = moduleOptions.flatMap((m) => m.categories)
      return [...new Set(all)]
    }
    return moduleOptions.find((m) => m.name === module)?.categories ?? []
  })

  function handleModuleChange(value: string) {
    if (value === '__all__') {
      module = null
      category = null
    } else {
      module = value
      category = null
    }
    onChange?.()
  }

  function handleCategoryChange(value: string) {
    category = value === '__all__' ? null : value
    onChange?.()
  }
</script>

<div class="flex items-center gap-2">
  <Select.Root
    type="single"
    value={module ?? '__all__'}
    onValueChange={handleModuleChange}
  >
    <Select.Trigger class="h-8 w-28">
      {module ?? '功能模块'}
    </Select.Trigger>
    <Select.Content>
      <Select.Item value="__all__">全部</Select.Item>
      {#each moduleOptions as opt (opt.name)}
        <Select.Item value={opt.name} label={opt.name}>
          {opt.name}
        </Select.Item>
      {/each}
    </Select.Content>
  </Select.Root>

  <Select.Root
    type="single"
    value={category ?? '__all__'}
    onValueChange={handleCategoryChange}
  >
    <Select.Trigger class="h-8 w-28">
      {category ?? '分类'}
    </Select.Trigger>
    <Select.Content>
      <Select.Item value="__all__">全部</Select.Item>
      {#each categoryOptions as cat (cat)}
        <Select.Item value={cat} label={cat}>{cat}</Select.Item>
      {/each}
    </Select.Content>
  </Select.Root>
</div>
