<script lang="ts" generics="TData extends RowData, TValue extends CellData">
  import { Badge } from '#lib/components/ui/badge/index.js'
  import { Button } from '#lib/components/ui/button/index.js'
  import * as Command from '#lib/components/ui/command/index.js'
  import * as Popover from '#lib/components/ui/popover/index.js'
  import { Separator } from '#lib/components/ui/separator/index.js'
  import { cn } from '#lib/utils.js'
  import { Check, CirclePlus } from '@lucide/svelte'
  import type { CellData, Column, RowData } from '@tanstack/table-core'
  import type { AppTableFeatures } from '#lib/table-features.js'

  let {
    column,
    title,
    options,
  }: {
    column?: Column<AppTableFeatures, TData, TValue>
    title?: string
    options: {
      label: string
      value: string
      icon?: import('svelte').ComponentType
    }[]
  } = $props()

  const facets = $derived(column?.getFacetedUniqueValues())
  const selectedValues = $derived.by(() => {
    const raw = column?.getFilterValue()
    return new Set(Array.isArray(raw) ? (raw as string[]) : [])
  })

  function handleSelect(value: string) {
    const newSelectedValues = new Set(selectedValues)
    if (newSelectedValues.has(value)) {
      newSelectedValues.delete(value)
    } else {
      newSelectedValues.add(value)
    }
    const filterValues = Array.from(newSelectedValues)
    column?.setFilterValue(filterValues.length ? filterValues : undefined)
  }
</script>

<Popover.Root>
  <Popover.Trigger>
    {#snippet child({ props })}
      <Button variant="outline" size="sm" class="h-8 border-dashed" {...props}>
        <CirclePlus class="mr-2 size-4" />
        {title}
        {#if selectedValues.size > 0}
          <Separator orientation="vertical" class="mx-2 h-4" />
          <Badge
            variant="secondary"
            class="rounded-sm px-1 font-normal lg:hidden"
          >
            {selectedValues.size}
          </Badge>
          <div class="hidden space-x-1 lg:flex">
            {#if selectedValues.size > 2}
              <Badge variant="secondary" class="rounded-sm px-1 font-normal">
                {selectedValues.size} selected
              </Badge>
            {:else}
              {#each options.filter( (opt) => selectedValues.has(opt.value) ) as option}
                <Badge variant="secondary" class="rounded-sm px-1 font-normal">
                  {option.label}
                </Badge>
              {/each}
            {/if}
          </div>
        {/if}
      </Button>
    {/snippet}
  </Popover.Trigger>
  <Popover.Content class="w-[200px] p-0" align="start">
    <Command.Root>
      <Command.Input placeholder={title} />
      <Command.List>
        <Command.Empty>No results found.</Command.Empty>
        <Command.Group>
          {#each options as option}
            {@const isSelected = selectedValues.has(option.value)}
            <Command.Item onSelect={() => handleSelect(option.value)}>
              <div
                class={cn(
                  'border-primary mr-2 flex size-4 items-center justify-center rounded-sm border',
                  isSelected
                    ? 'bg-primary text-primary-foreground'
                    : 'opacity-50 [&_svg]:invisible'
                )}
              >
                <Check class="size-4" />
              </div>
              {#if option.icon}
                <option.icon class="text-muted-foreground mr-2 size-4" />
              {/if}
              <span>{option.label}</span>
              {#if facets?.has(option.value)}
                <span
                  class="ml-auto flex size-4 items-center justify-center font-mono text-xs"
                >
                  {facets.get(option.value)}
                </span>
              {/if}
            </Command.Item>
          {/each}
        </Command.Group>
        {#if selectedValues.size > 0}
          <Command.Separator />
          <Command.Group>
            <Command.Item
              onSelect={() => column?.setFilterValue(undefined)}
              class="justify-center text-center"
            >
              Clear filters
            </Command.Item>
          </Command.Group>
        {/if}
      </Command.List>
    </Command.Root>
  </Popover.Content>
</Popover.Root>
