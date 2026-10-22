<script lang="ts">
  import { goto } from '$app/navigation'
  import { sidebarData } from '$lib/components/layout/data/sidebar-data.js'
  import * as Command from '$lib/components/ui/command/index.js'
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js'
  import { useSearch } from '$lib/context/search.svelte.js'
  import { useTheme } from '$lib/context/theme.svelte.js'
  import { COLOR_THEMES } from '$lib/themes/registry.js'
  import { ChevronRight, Laptop, Moon, Sun } from '@lucide/svelte'

  const search = useSearch()
  const theme = useTheme()

  function runCommand(command: () => void) {
    search.setOpen(false)
    command()
  }
</script>

<Command.Dialog open={search.open} onOpenChange={search.setOpen}>
  <Command.Input placeholder="输入命令或搜索..." />
  <Command.List>
    <ScrollArea type="hover" class="h-72 pe-1">
      <Command.Empty>未找到结果。</Command.Empty>
      {#each sidebarData.navGroups as group}
        <Command.Group heading={group.title}>
          {#each group.items as navItem}
            {#if navItem.url}
              <Command.Item
                value={`${navItem.title} ${navItem.url}`}
                onSelect={() => runCommand(() => goto(navItem.url))}
              >
                {#if navItem.icon}
                  {@const Icon = navItem.icon}
                  <Icon />
                {/if}
                <span>{navItem.title}</span>
              </Command.Item>
            {:else if navItem.items}
              {#each navItem.items as subItem}
                <Command.Item
                  value={`${navItem.title} ${subItem.title} ${subItem.url}`}
                  onSelect={() => runCommand(() => goto(subItem.url))}
                >
                  {#if subItem.icon}
                    {@const SubIcon = subItem.icon}
                    <SubIcon />
                  {:else if navItem.icon}
                    {@const ParentIcon = navItem.icon}
                    <ParentIcon />
                  {/if}
                  <span>
                    {navItem.title}
                    <ChevronRight class="inline-block size-3" />
                    {subItem.title}
                  </span>
                </Command.Item>
              {/each}
            {/if}
          {/each}
        </Command.Group>
      {/each}
      <Command.Separator />
      <Command.Group heading="颜色主题">
        {#each COLOR_THEMES as ct}
          <Command.Item
            onSelect={() => runCommand(() => theme.setColorTheme(ct.name))}
          >
            <span>{ct.label}</span>
          </Command.Item>
        {/each}
      </Command.Group>
      <Command.Group heading="外观模式">
        <Command.Item
          onSelect={() => runCommand(() => theme.setTheme('light'))}
        >
          <Sun class="mr-2 size-4" /> <span>浅色</span>
        </Command.Item>
        <Command.Item onSelect={() => runCommand(() => theme.setTheme('dark'))}>
          <Moon class="mr-2 size-4 scale-90" /> <span>深色</span>
        </Command.Item>
        <Command.Item
          onSelect={() => runCommand(() => theme.setTheme('system'))}
        >
          <Laptop class="mr-2 size-4" /> <span>跟随系统</span>
        </Command.Item>
      </Command.Group>
    </ScrollArea>
  </Command.List>
</Command.Dialog>
