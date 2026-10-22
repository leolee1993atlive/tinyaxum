<script lang="ts">
  import { untrack } from 'svelte'
  import type { SqlQueryParamFieldVo } from '$lib/api/types.js'
  import { Button } from '$lib/components/ui/button/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import * as Popover from '$lib/components/ui/popover/index.js'
  import { Settings2 } from '@lucide/svelte'

  interface Props {
    paramFields: SqlQueryParamFieldVo[]
    globalParamNames: string[]
    defaults: Record<string, string>
    onChange: (next: Record<string, string>) => void
  }

  let { paramFields, globalParamNames, defaults, onChange }: Props = $props()

  let open = $state(false)
  let local = $state<Record<string, string>>(untrack(() => ({ ...defaults })))

  $effect(() => {
    if (open) local = { ...defaults }
  })

  function commit() {
    const cleaned: Record<string, string> = {}
    for (const [k, v] of Object.entries(local)) {
      if (v !== '' && v !== undefined && v !== null) cleaned[k] = v
    }
    onChange(cleaned)
    open = false
  }

  function reset() {
    local = {}
  }
</script>

<Popover.Root bind:open>
  <Popover.Trigger>
    {#snippet child({ props })}
      <Button
        variant="ghost"
        size="icon"
        class="size-8 shrink-0"
        type="button"
        aria-label="参数默认值"
        title="参数默认值"
        {...props}
      >
        <Settings2 class="size-4" />
      </Button>
    {/snippet}
  </Popover.Trigger>
  <Popover.Content class="w-80 space-y-3 p-3" align="end">
    <p class="text-sm font-medium">参数默认值</p>
    {#if paramFields.length === 0}
      <p class="text-muted-foreground text-xs">该查询未声明参数。</p>
    {:else}
      <div class="space-y-2">
        {#each paramFields as p (p.fieldName)}
          {@const isGlobal = globalParamNames.includes(p.fieldName)}
          <div class="space-y-1">
            <label
              class="text-muted-foreground text-xs"
              for={`def-${p.fieldName}`}
            >
              {p.commentLabel || p.fieldName}{p.required ? ' *' : ''}
            </label>
            <Input
              id={`def-${p.fieldName}`}
              value={isGlobal ? '' : (local[p.fieldName] ?? '')}
              placeholder={isGlobal ? '(全局)' : ''}
              disabled={isGlobal}
              oninput={(e) =>
                (local = {
                  ...local,
                  [p.fieldName]: e.currentTarget.value,
                })}
              class="h-8 w-full"
            />
          </div>
        {/each}
      </div>
    {/if}
    <div class="flex items-center justify-between border-t pt-2">
      <Button type="button" variant="ghost" size="sm" onclick={reset}>
        重置
      </Button>
      <Button type="button" variant="default" size="sm" onclick={commit}>
        完成
      </Button>
    </div>
  </Popover.Content>
</Popover.Root>
