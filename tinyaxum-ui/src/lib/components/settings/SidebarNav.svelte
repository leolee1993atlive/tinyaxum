<script lang="ts">
  import type { Component } from 'svelte'
  import { goto } from '$app/navigation'
  import { page } from '$app/state'
  import { buttonVariants } from '#lib/components/ui/button/index.js'
  import { ScrollArea } from '#lib/components/ui/scroll-area/index.js'
  import { cn } from '#lib/utils.js'

  interface NavItem {
    href: string
    title: string
    icon: Component<{ size?: number }>
  }

  interface Props {
    items: NavItem[]
    class?: string
  }

  let { items, class: className }: Props = $props()

  let pathname = $derived(page.url.pathname)

  function handleSelect(value: string) {
    goto(value)
  }
</script>

<div class="p-1 md:hidden">
  <select
    class="border-input bg-background flex h-12 w-full rounded-md border px-3 py-2 text-sm sm:w-48"
    value={pathname}
    onchange={(event) => handleSelect(event.currentTarget.value)}
  >
    {#each items as item}
      <option value={item.href}>{item.title}</option>
    {/each}
  </select>
</div>

<ScrollArea
  orientation="horizontal"
  class="bg-background hidden w-full min-w-40 px-1 py-2 md:block"
>
  <nav
    class={cn(
      'flex space-x-2 py-1 lg:flex-col lg:space-y-1 lg:space-x-0',
      className
    )}
  >
    {#each items as item}
      {@const Icon = item.icon}
      <a
        href={item.href}
        class={cn(
          buttonVariants({ variant: 'ghost' }),
          pathname === item.href
            ? 'bg-muted hover:bg-accent'
            : 'hover:bg-accent hover:underline',
          'justify-start'
        )}
      >
        <span class="me-2">
          <Icon size={18} />
        </span>
        {item.title}
      </a>
    {/each}
  </nav>
</ScrollArea>
