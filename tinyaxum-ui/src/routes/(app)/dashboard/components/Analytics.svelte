<script lang="ts">
  import * as Card from '$lib/components/ui/card/index.js'
  import AnalyticsChart from './AnalyticsChart.svelte'

  const referrers = [
    { name: 'Direct', value: 512 },
    { name: 'Product Hunt', value: 238 },
    { name: 'Twitter', value: 174 },
    { name: 'Blog', value: 104 },
  ]

  const devices = [
    { name: 'Desktop', value: 74 },
    { name: 'Mobile', value: 22 },
    { name: 'Tablet', value: 4 },
  ]
</script>

{#snippet SimpleBarList({
  items,
  valueFormatter,
  barClass,
}: {
  items: { name: string; value: number }[]
  valueFormatter: (n: number) => string
  barClass: string
})}
  {@const max = Math.max(...items.map((i) => i.value), 1)}
  <ul class="space-y-3">
    {#each items as item}
      {@const width = `${Math.round((item.value / max) * 100)}%`}
      <li class="flex items-center justify-between gap-3">
        <div class="min-w-0 flex-1">
          <div class="text-muted-foreground mb-1 truncate text-xs">
            {item.name}
          </div>
          <div class="bg-muted h-2.5 w-full rounded-full">
            <div class="h-2.5 rounded-full {barClass}" style:width></div>
          </div>
        </div>
        <div class="ps-2 text-xs font-medium tabular-nums">
          {valueFormatter(item.value)}
        </div>
      </li>
    {/each}
  </ul>
{/snippet}

<div class="space-y-4">
  <Card.Root>
    <Card.Header>
      <Card.Title>流量概览</Card.Title>
      <Card.Description>每周点击量和独立访客</Card.Description>
    </Card.Header>
    <Card.Content class="px-6">
      <AnalyticsChart />
    </Card.Content>
  </Card.Root>

  <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">总点击量</Card.Title>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          class="text-muted-foreground h-4 w-4"
        >
          <path d="M3 3v18h18" />
          <path d="M7 15l4-4 4 4 4-6" />
        </svg>
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">1,248</div>
        <p class="text-muted-foreground text-xs">较上周 +12.4%</p>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">独立访客</Card.Title>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          class="text-muted-foreground h-4 w-4"
        >
          <circle cx="12" cy="7" r="4" />
          <path d="M6 21v-2a6 6 0 0 1 12 0v2" />
        </svg>
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">832</div>
        <p class="text-muted-foreground text-xs">较上周 +5.8%</p>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">跳出率</Card.Title>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          class="text-muted-foreground h-4 w-4"
        >
          <path d="M3 12h6l3 6 3-6h6" />
        </svg>
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">42%</div>
        <p class="text-muted-foreground text-xs">较上周 -3.2%</p>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <Card.Title class="text-sm font-medium">平均会话时长</Card.Title>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          class="text-muted-foreground h-4 w-4"
        >
          <circle cx="12" cy="12" r="10" />
          <path d="M12 6v6l4 2" />
        </svg>
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">3m 24s</div>
        <p class="text-muted-foreground text-xs">较上周 +18秒</p>
      </Card.Content>
    </Card.Root>
  </div>

  <div class="grid grid-cols-1 gap-4 lg:grid-cols-7">
    <Card.Root class="col-span-1 lg:col-span-4">
      <Card.Header>
        <Card.Title>来源</Card.Title>
        <Card.Description>主要流量来源</Card.Description>
      </Card.Header>
      <Card.Content>
        {@render SimpleBarList({
          items: referrers,
          barClass: 'bg-primary',
          valueFormatter: (n) => `${n}`,
        })}
      </Card.Content>
    </Card.Root>

    <Card.Root class="col-span-1 lg:col-span-3">
      <Card.Header>
        <Card.Title>设备</Card.Title>
        <Card.Description>用户如何访问您的应用</Card.Description>
      </Card.Header>
      <Card.Content>
        {@render SimpleBarList({
          items: devices,
          barClass: 'bg-muted-foreground',
          valueFormatter: (n) => `${n}%`,
        })}
      </Card.Content>
    </Card.Root>
  </div>
</div>
