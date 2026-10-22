<script lang="ts">
  import { Button } from '$lib/components/ui/button/index.js'
  import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js'
  import { cn } from '$lib/utils.js'
  import { Menu } from '@lucide/svelte'

  interface Link {
    title: string
    href: string
    isActive: boolean
    disabled?: boolean
  }

  let {
    class: className,
    links,
    ...props
  } = $props<{
    class?: string
    links: Link[]
    [key: string]: unknown
  }>()
</script>

<div class="lg:hidden">
  <DropdownMenu.Root>
    <DropdownMenu.Trigger>
      {#snippet child({ props }: { props: Record<string, unknown> })}
        <Button {...props} size="icon" variant="outline" class="md:size-7">
          <Menu />
        </Button>
      {/snippet}
    </DropdownMenu.Trigger>
    <DropdownMenu.Content side="bottom" align="start">
      {#each links as { title, href, isActive, disabled }}
        <DropdownMenu.Item {disabled}>
          <a
            href={disabled ? undefined : href}
            class={cn(
              'flex w-full items-center py-1',
              !isActive && 'text-muted-foreground',
              disabled && 'cursor-not-allowed opacity-50'
            )}
          >
            {title}
          </a>
        </DropdownMenu.Item>
      {/each}
    </DropdownMenu.Content>
  </DropdownMenu.Root>
</div>

<nav
  class={cn(
    'hidden items-center space-x-4 lg:flex lg:space-x-4 xl:space-x-6',
    className
  )}
  {...props}
>
  {#each links as { title, href, isActive, disabled }}
    <a
      href={disabled ? undefined : href}
      class={cn(
        'hover:text-primary text-sm font-medium transition-colors',
        !isActive && 'text-muted-foreground',
        disabled && 'pointer-events-none opacity-50'
      )}
    >
      {title}
    </a>
  {/each}
</nav>
