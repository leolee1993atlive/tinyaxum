<script lang="ts">
  import { Button } from '#lib/components/ui/button/index.js'
  import * as DropdownMenu from '#lib/components/ui/dropdown-menu/index.js'
  import { cn } from '#lib/utils.js'
  import { LoaderCircle, ChevronDown } from '@lucide/svelte'

  interface Props {
    value?: string | undefined
    onValueChange?: (value: string) => void
    placeholder?: string
    isPending?: boolean
    items: { label: string; value: string }[] | undefined
    disabled?: boolean
    class?: string
  }

  let {
    value = $bindable(undefined),
    onValueChange,
    placeholder = 'Select',
    isPending = false,
    items = [],
    disabled = false,
    class: className = '',
  }: Props = $props()

  const selectedLabel = $derived(
    items?.find((item) => item.value === value)?.label ?? placeholder
  )

  function handleSelect(val: string) {
    value = val
    onValueChange?.(val)
  }
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger disabled={disabled || isPending}>
    {#snippet child({ props })}
      <Button
        variant="outline"
        class={cn(
          'flex w-full items-center justify-between px-3 font-normal',
          className
        )}
        {...props}
      >
        <span class={cn(!value && 'text-muted-foreground')}>
          {selectedLabel}
        </span>
        <ChevronDown class="h-4 w-4 opacity-50" />
      </Button>
    {/snippet}
  </DropdownMenu.Trigger>
  <DropdownMenu.Content
    class="min-w-[var(--bits-dropdown-menu-anchor-width)] p-1"
  >
    {#if isPending}
      <DropdownMenu.Item disabled class="h-14 justify-center gap-2">
        <LoaderCircle class="h-5 w-5 animate-spin" />
        Loading...
      </DropdownMenu.Item>
    {:else if items && items.length > 0}
      {#each items as item (item.value)}
        <DropdownMenu.Item
          onclick={() => handleSelect(item.value)}
          class={cn(value === item.value && 'bg-accent text-accent-foreground')}
        >
          {item.label}
        </DropdownMenu.Item>
      {/each}
    {:else}
      <DropdownMenu.Item disabled>No items found</DropdownMenu.Item>
    {/if}
  </DropdownMenu.Content>
</DropdownMenu.Root>
