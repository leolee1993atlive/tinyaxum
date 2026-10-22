<script lang="ts">
  const chartData = [
    { name: '1月', total: 2400 },
    { name: '2月', total: 1398 },
    { name: '3月', total: 9800 },
    { name: '4月', total: 3908 },
    { name: '5月', total: 4800 },
    { name: '6月', total: 3800 },
    { name: '7月', total: 4300 },
    { name: '8月', total: 5100 },
    { name: '9月', total: 4700 },
    { name: '10月', total: 6200 },
    { name: '11月', total: 5900 },
    { name: '12月', total: 7100 },
  ]

  const maxValue = Math.max(...chartData.map((item) => item.total), 1)
</script>

<div class="space-y-4">
  <div class="flex h-[300px] items-end gap-2">
    {#each chartData as item}
      <div class="flex min-w-0 flex-1 flex-col items-center gap-2">
        <div class="bg-muted flex h-full w-full items-end rounded-md px-1 pb-1">
          <div
            class="bg-primary w-full rounded-sm"
            style:height={`${Math.round((item.total / maxValue) * 100)}%`}
          ></div>
        </div>
        <span class="text-muted-foreground text-xs">{item.name}</span>
      </div>
    {/each}
  </div>

  <div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
    {#each chartData.slice(-4) as item}
      <div class="bg-muted/50 rounded-md px-3 py-2">
        <div class="text-muted-foreground text-xs">{item.name}</div>
        <div class="text-sm font-medium">${item.total.toLocaleString()}</div>
      </div>
    {/each}
  </div>
</div>
