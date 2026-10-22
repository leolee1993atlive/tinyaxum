<script lang="ts">
  const chartData = [
    { name: 'Mon', clicks: 340, uniques: 210 },
    { name: 'Tue', clicks: 420, uniques: 280 },
    { name: 'Wed', clicks: 390, uniques: 250 },
    { name: 'Thu', clicks: 510, uniques: 320 },
    { name: 'Fri', clicks: 610, uniques: 410 },
    { name: 'Sat', clicks: 480, uniques: 300 },
    { name: 'Sun', clicks: 450, uniques: 295 },
  ]

  const maxValue = Math.max(...chartData.map((item) => item.clicks), 1)
</script>

<div class="space-y-4">
  <div class="grid grid-cols-7 gap-2">
    {#each chartData as item}
      <div class="flex flex-col items-center gap-2">
        <div
          class="bg-primary/15 flex h-48 w-full items-end rounded-md px-1 pb-1"
        >
          <div
            class="bg-primary w-full rounded-sm"
            style:height={`${Math.round((item.clicks / maxValue) * 100)}%`}
          ></div>
        </div>
        <div class="text-muted-foreground text-xs">{item.name}</div>
      </div>
    {/each}
  </div>

  <div class="grid gap-2 sm:grid-cols-2">
    {#each chartData as item}
      <div
        class="bg-muted/50 flex items-center justify-between rounded-md px-3 py-2"
      >
        <span class="text-sm font-medium">{item.name}</span>
        <span class="text-muted-foreground text-xs">
          {item.clicks} clicks / {item.uniques} uniques
        </span>
      </div>
    {/each}
  </div>
</div>
