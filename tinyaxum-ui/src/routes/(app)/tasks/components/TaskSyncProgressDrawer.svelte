<script lang="ts">
  import { Button } from '$lib/components/ui/button/index.js'
  import * as Sheet from '$lib/components/ui/sheet/index.js'
  import { CheckCircle, Hash, LoaderCircle, XCircle, Zap } from '@lucide/svelte'
  import { useTasks } from './tasks-state.svelte'
  import type { SyncPhase } from './tasks-state.svelte'

  const tasks = useTasks()

  const isOpen = $derived(tasks.open === 'syncProgress')
  const task = $derived(tasks.currentRow)
  const progress = $derived(task ? tasks.getProgress(task.transTaskId) : null)

  const phase = $derived<SyncPhase>(progress?.phase ?? 'exporting')
  const rowsDone = $derived(progress?.rowsDone ?? null)
  const doneRows = $derived(progress?.doneRows ?? null)
  const newWatermark = $derived(progress?.newWatermark ?? null)
  const errorMessage = $derived(progress?.errorMessage ?? null)
  const elapsedSeconds = $derived(progress?.elapsedSeconds ?? 0)

  const isDone = $derived(phase === 'done' || phase === 'error')

  const phaseLabel: Record<SyncPhase, string> = {
    exporting: '正在写入数据…',
    done: '同步完成',
    error: '同步失败',
  }

  function formatSeconds(s: number): string {
    if (s < 60) return `${s}s`
    const m = Math.floor(s / 60)
    const rem = s % 60
    return `${m}m ${rem}s`
  }

  let closeTimeoutHandle: ReturnType<typeof setTimeout> | null = null

  function cancelPendingClose() {
    if (closeTimeoutHandle !== null) {
      clearTimeout(closeTimeoutHandle)
      closeTimeoutHandle = null
    }
  }

  function handleClose() {
    tasks.setOpen(null)
    closeTimeoutHandle = setTimeout(() => {
      closeTimeoutHandle = null
      tasks.setCurrentRow(null)
    }, 200)
  }

  $effect(() => {
    if (isOpen) cancelPendingClose()
  })
</script>

<Sheet.Root
  open={isOpen}
  onOpenChange={(open) => {
    if (!open) handleClose()
  }}
>
  <Sheet.Content class="flex flex-col gap-0 sm:max-w-md">
    <Sheet.Header class="border-b px-6 py-4 text-start">
      <Sheet.Title class="flex items-center gap-2">
        {#if phase === 'done'}
          <CheckCircle class="size-5 text-green-500" />
        {:else if phase === 'error'}
          <XCircle class="text-destructive size-5" />
        {:else}
          <LoaderCircle class="text-primary size-5 animate-spin" />
        {/if}
        同步进度
      </Sheet.Title>
      <Sheet.Description class="truncate text-xs">
        {task?.tableName ?? '—'}
        {#if task?.tableComment}
          · {task.tableComment}
        {/if}
      </Sheet.Description>
    </Sheet.Header>

    <!-- 主内容 -->
    <div class="flex-1 space-y-6 overflow-y-auto px-6 py-6">
      <!-- 阶段指示器 -->
      <div class="flex flex-col gap-3">
        {#each ['exporting', 'done'] as SyncPhase[] as p}
          {@const isCurrent = phase === p && p !== 'done'}
          {@const isPassed =
            (p === 'exporting' && ['done', 'error'].includes(phase)) ||
            (p === 'done' && phase === 'done')}
          <div class="flex items-center gap-3">
            <div
              class={[
                'flex size-8 shrink-0 items-center justify-center rounded-full border-2 transition-colors',
                isPassed && p !== 'done'
                  ? 'border-green-500 bg-green-50 text-green-600'
                  : p === 'done' && phase === 'done'
                    ? 'border-green-500 bg-green-50 text-green-600'
                    : isCurrent
                      ? 'border-primary bg-primary/10 text-primary'
                      : phase === 'error'
                        ? 'border-muted text-muted-foreground'
                        : 'border-muted text-muted-foreground',
              ].join(' ')}
            >
              {#if p === 'exporting'}
                <Zap class="size-4" />
              {:else}
                <CheckCircle class="size-4" />
              {/if}
            </div>
            <div class="min-w-0 flex-1">
              <p
                class={[
                  'text-sm font-medium',
                  isCurrent
                    ? 'text-foreground'
                    : isPassed
                      ? 'text-muted-foreground'
                      : 'text-muted-foreground/60',
                ].join(' ')}
              >
                {#if p === 'exporting'}
                  写入数据
                {:else}
                  完成
                {/if}
              </p>
              {#if isCurrent}
                <p class="text-muted-foreground animate-pulse text-xs">
                  {phaseLabel[phase]}
                </p>
              {/if}
            </div>
            {#if isCurrent}
              <LoaderCircle class="text-primary size-4 shrink-0 animate-spin" />
            {:else if isPassed}
              <CheckCircle class="size-4 shrink-0 text-green-500" />
            {/if}
          </div>
        {/each}
      </div>

      <!-- 进度条（滚动样式） -->
      {#if phase === 'exporting'}
        <div class="space-y-1.5">
          <div class="text-muted-foreground flex justify-between text-xs">
            <span>已导出 {(rowsDone ?? 0).toLocaleString()} 行</span>
          </div>
          <div class="bg-muted h-2 overflow-hidden rounded-full">
            <div
              class="bg-primary progress-indeterminate h-full w-1/3 rounded-full"
            ></div>
          </div>
        </div>
      {/if}

      <!-- 统计信息 -->
      <div class="grid grid-cols-2 gap-3">
        {#if doneRows !== null}
          <div class="rounded-lg border p-3">
            <p class="text-muted-foreground mb-1 text-xs">同步行数</p>
            <p class="font-mono text-sm font-semibold text-green-600">
              {doneRows.toLocaleString()}
            </p>
          </div>
        {/if}
        <div class="rounded-lg border p-3">
          <p class="text-muted-foreground mb-1 text-xs">耗时</p>
          <p class="font-mono text-sm font-semibold">
            {formatSeconds(elapsedSeconds)}
          </p>
        </div>
        {#if newWatermark}
          <div class="col-span-2 rounded-lg border p-3">
            <p
              class="text-muted-foreground mb-1 flex items-center gap-1 text-xs"
            >
              <Hash class="size-3" /> 新水位值
            </p>
            <p class="truncate font-mono text-xs">{newWatermark}</p>
          </div>
        {/if}
      </div>

      <!-- 错误信息 -->
      {#if phase === 'error' && errorMessage}
        <div
          class="bg-destructive/10 border-destructive/20 rounded-lg border px-4 py-3"
        >
          <p class="text-destructive mb-1 text-sm font-medium">同步失败</p>
          <p class="text-destructive/80 text-xs break-all">{errorMessage}</p>
        </div>
      {/if}
    </div>

    <!-- 底部操作 -->
    <Sheet.Footer class="border-t px-6 py-4">
      <Button
        variant={phase === 'error' ? 'destructive' : 'default'}
        disabled={!isDone}
        onclick={handleClose}
        class="w-full"
      >
        {#if !isDone}
          <LoaderCircle class="mr-2 size-4 animate-spin" />
          同步中，请稍候…
        {:else if phase === 'done'}
          完成
        {:else}
          关闭
        {/if}
      </Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>

<style>
  .progress-indeterminate {
    animation: progress-indeterminate-slide 1.6s ease-in-out infinite;
  }

  @keyframes progress-indeterminate-slide {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(400%);
    }
  }
</style>
