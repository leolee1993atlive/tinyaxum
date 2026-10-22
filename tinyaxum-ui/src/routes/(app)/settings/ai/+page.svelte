<script lang="ts">
  import { onMount } from 'svelte'
  import { aiProvidersApi } from '$lib/api/ai-providers'
  import type { AiProviderListVo } from '$lib/api/types'
  import { Button } from '$lib/components/ui/button/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import { Label } from '$lib/components/ui/label/index.js'
  import * as Sheet from '$lib/components/ui/sheet/index.js'
  import * as Table from '$lib/components/ui/table/index.js'
  import { Textarea } from '$lib/components/ui/textarea/index.js'
  import { Star, Plus } from '@lucide/svelte'
  import { toast } from 'svelte-sonner'

  let providers = $state<AiProviderListVo[]>([])
  let loading = $state(false)

  type FormMode = 'create' | { mode: 'edit'; id: number }
  let sheetOpen = $state(false)
  let formMode = $state<FormMode>('create')
  let form = $state({
    name: '',
    baseUrl: '',
    apiKey: '',
    model: '',
    systemPrompt:
      '你是一名资深数据分析师,擅长将 MySQL SQL 改写为 DuckDB SQL,精简多余 select 字段,并保留语义等价。',
    temperature: 0.3,
  })

  async function refresh() {
    loading = true
    try {
      const res = await aiProvidersApi.list()
      providers = res.data ?? []
    } catch (e) {
      toast.error(`加载失败: ${e}`)
    } finally {
      loading = false
    }
  }

  async function handleSetDefault(id: number) {
    try {
      await aiProvidersApi.setDefault(id)
      toast.success('已设为默认')
      await refresh()
    } catch (e) {
      toast.error(`操作失败: ${e}`)
    }
  }

  async function handleTest(id: number) {
    try {
      const res = await aiProvidersApi.test(id)
      const r = res.data
      if (r?.ok) {
        toast.success(
          `连接成功 ${r.latencyMs}ms${r.modelEcho ? ` · ${r.modelEcho}` : ''}`
        )
      } else {
        toast.error(`连接失败: ${r?.error ?? '未知错误'}`)
      }
    } catch (e) {
      toast.error(`测试失败: ${e}`)
    }
  }

  async function handleDelete(id: number) {
    if (
      !confirm(
        '确定要删除此 provider 吗?引用此 provider 的会话仍可看历史,但发新消息会报错。'
      )
    ) {
      return
    }
    try {
      await aiProvidersApi.delete(id)
      toast.success('已删除')
      await refresh()
    } catch (e) {
      toast.error(`删除失败: ${e}`)
    }
  }

  function openCreate() {
    formMode = 'create'
    form = {
      name: '',
      baseUrl: '',
      apiKey: '',
      model: '',
      systemPrompt: form.systemPrompt,
      temperature: 0.3,
    }
    sheetOpen = true
  }

  async function openEdit(p: AiProviderListVo) {
    formMode = { mode: 'edit', id: p.aiProviderId }
    form = {
      name: p.name,
      baseUrl: p.baseUrl,
      apiKey: '',
      model: p.model,
      systemPrompt: p.systemPrompt,
      temperature: p.temperature,
    }
    sheetOpen = true
  }

  async function handleSave() {
    try {
      if (formMode === 'create') {
        if (!form.apiKey) {
          toast.error('新建时必须填写 API Key')
          return
        }
        await aiProvidersApi.create(form)
      } else {
        await aiProvidersApi.update(formMode.id, form)
      }
      toast.success('保存成功')
      sheetOpen = false
      await refresh()
    } catch (e) {
      toast.error(`保存失败: ${e}`)
    }
  }

  onMount(refresh)
</script>

<div class="flex w-full flex-col gap-4">
  <div class="flex items-center justify-between">
    <h2 class="text-xl font-semibold">AI 模型供应商</h2>
    <Button onclick={openCreate}>
      <Plus class="mr-2 h-4 w-4" />
      新增
    </Button>
  </div>

  <Table.Root>
    <Table.Header>
      <Table.Row>
        <Table.Head class="w-12">默认</Table.Head>
        <Table.Head>名称</Table.Head>
        <Table.Head>模型</Table.Head>
        <Table.Head>Base URL</Table.Head>
        <Table.Head>API Key</Table.Head>
        <Table.Head class="w-64 text-right">操作</Table.Head>
      </Table.Row>
    </Table.Header>
    <Table.Body>
      {#if loading}
        <Table.Row><Table.Cell colspan={6}>加载中...</Table.Cell></Table.Row>
      {:else if providers.length === 0}
        <Table.Row
          ><Table.Cell colspan={6} class="text-muted-foreground text-center"
            >暂无 provider,请新增</Table.Cell
          ></Table.Row
        >
      {:else}
        {#each providers as p (p.aiProviderId)}
          <Table.Row>
            <Table.Cell>
              <button
                type="button"
                aria-label={p.isDefault ? '当前默认' : '设为默认'}
                onclick={() => handleSetDefault(p.aiProviderId)}
              >
                <Star
                  class={p.isDefault
                    ? 'h-4 w-4 fill-yellow-400 text-yellow-400'
                    : 'text-muted-foreground h-4 w-4'}
                />
              </button>
            </Table.Cell>
            <Table.Cell class="font-medium">{p.name}</Table.Cell>
            <Table.Cell>{p.model}</Table.Cell>
            <Table.Cell class="text-muted-foreground text-xs"
              >{p.baseUrl}</Table.Cell
            >
            <Table.Cell class="font-mono text-xs">{p.apiKeyMasked}</Table.Cell>
            <Table.Cell class="space-x-2 text-right">
              <Button
                size="sm"
                variant="outline"
                onclick={() => handleTest(p.aiProviderId)}>测试</Button
              >
              <Button size="sm" variant="outline" onclick={() => openEdit(p)}
                >编辑</Button
              >
              <Button
                size="sm"
                variant="destructive"
                onclick={() => handleDelete(p.aiProviderId)}>删除</Button
              >
            </Table.Cell>
          </Table.Row>
        {/each}
      {/if}
    </Table.Body>
  </Table.Root>
</div>

<Sheet.Root bind:open={sheetOpen}>
  <Sheet.Content side="right" class="flex w-full flex-col sm:max-w-md">
    <Sheet.Header>
      <Sheet.Title
        >{formMode === 'create' ? '新增供应商' : '编辑供应商'}</Sheet.Title
      >
    </Sheet.Header>
    <div class="flex-1 space-y-4 overflow-y-auto px-4 py-4">
      <div class="grid gap-2">
        <Label for="name">名称</Label>
        <Input id="name" bind:value={form.name} />
      </div>
      <div class="grid gap-2">
        <Label for="baseUrl">Base URL</Label>
        <Input
          id="baseUrl"
          bind:value={form.baseUrl}
          placeholder="https://api.deepseek.com/v1"
        />
      </div>
      <div class="grid gap-2">
        <Label for="apiKey">API Key</Label>
        <Input
          id="apiKey"
          type="password"
          bind:value={form.apiKey}
          placeholder={formMode === 'create' ? 'sk-...' : '留空保持不变'}
        />
      </div>
      <div class="grid gap-2">
        <Label for="model">模型</Label>
        <Input id="model" bind:value={form.model} placeholder="deepseek-chat" />
      </div>
      <div class="grid gap-2">
        <Label for="temp">Temperature</Label>
        <Input
          id="temp"
          type="number"
          step="0.1"
          min="0"
          max="2"
          bind:value={form.temperature}
        />
      </div>
      <div class="grid gap-2">
        <Label for="prompt">System Prompt</Label>
        <Textarea id="prompt" rows={6} bind:value={form.systemPrompt} />
      </div>
    </div>
    <Sheet.Footer>
      <Button variant="ghost" onclick={() => (sheetOpen = false)}>取消</Button>
      <Button onclick={handleSave}>保存</Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>
