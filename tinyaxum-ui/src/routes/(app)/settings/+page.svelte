<script lang="ts">
  import { onMount } from 'svelte'
  import { usersApi } from '$lib/api/users.js'
  import ContentSection from '$lib/components/settings/ContentSection.svelte'
  import type { User } from '$lib/data/users/schema.js'

  let user = $state<User | null>(null)
  let username = $state('')
  let nickName = $state('')
  let email = $state('')
  let isLoading = $state(false)
  let isSaving = $state(false)
  let error = $state<string | null>(null)
  let success = $state<string | null>(null)

  onMount(async () => {
    isLoading = true
    error = null
    try {
      const currentResponse = await usersApi.getCurrentUser()
      if (currentResponse.data) {
        const fullResponse = await usersApi.getUser(currentResponse.data.userId)
        if (fullResponse.data) {
          user = fullResponse.data
          username = fullResponse.data.username
          nickName = fullResponse.data.nickName
          email = fullResponse.data.email || ''
        }
      }
    } catch (err) {
      error = err instanceof Error ? err.message : '加载个人资料失败'
    } finally {
      isLoading = false
    }
  })

  async function handleSave() {
    if (!user) return

    isSaving = true
    error = null
    success = null

    try {
      await usersApi.updateUser({
        userId: user.userId,
        username,
        nickName,
        email: email || undefined,
        active: user.active,
      })
      success = '个人资料更新成功'
    } catch (err) {
      error = err instanceof Error ? err.message : '更新个人资料失败'
    } finally {
      isSaving = false
    }
  }

  function dismissMessage() {
    error = null
    success = null
  }
</script>

<ContentSection title="个人资料" desc="这是其他人在网站上看到您的方式。">
  {#if isLoading}
    <div class="text-muted-foreground py-8 text-center">加载中...</div>
  {:else}
    <div class="space-y-6">
      {#if error}
        <div
          class="flex items-center justify-between rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-800"
        >
          <span>{error}</span>
          <button
            type="button"
            onclick={dismissMessage}
            class="text-red-600 hover:text-red-800"
          >
            ✕
          </button>
        </div>
      {/if}

      {#if success}
        <div
          class="flex items-center justify-between rounded-md border border-green-200 bg-green-50 px-4 py-3 text-sm text-green-800"
        >
          <span>{success}</span>
          <button
            type="button"
            onclick={dismissMessage}
            class="text-green-600 hover:text-green-800"
          >
            ✕
          </button>
        </div>
      {/if}

      <div class="space-y-2">
        <label class="text-sm font-medium" for="username">用户名</label>
        <input
          id="username"
          type="text"
          bind:value={username}
          class="border-input placeholder:text-muted-foreground focus-visible:ring-ring flex h-9 w-full rounded-md border bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium focus-visible:ring-1 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
        />
        <p class="text-muted-foreground text-sm">
          这是您的公开显示名称。可以是您的真实姓名或化名。
        </p>
      </div>

      <div class="space-y-2">
        <label class="text-sm font-medium" for="nickName">显示名称</label>
        <input
          id="nickName"
          type="text"
          bind:value={nickName}
          class="border-input placeholder:text-muted-foreground focus-visible:ring-ring flex h-9 w-full rounded-md border bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium focus-visible:ring-1 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
        />
        <p class="text-muted-foreground text-sm">界面中显示的友好名称。</p>
      </div>

      <div class="space-y-2">
        <label class="text-sm font-medium" for="email">邮箱</label>
        <input
          id="email"
          type="email"
          bind:value={email}
          class="border-input placeholder:text-muted-foreground focus-visible:ring-ring flex h-9 w-full rounded-md border bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium focus-visible:ring-1 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
        />
        <p class="text-muted-foreground text-sm">
          您可以在邮箱设置中管理已验证的邮箱地址。
        </p>
      </div>

      <button
        type="button"
        onclick={handleSave}
        disabled={isSaving}
        class="bg-primary text-primary-foreground hover:bg-primary/90 focus-visible:ring-ring inline-flex h-9 items-center justify-center rounded-md px-4 py-2 text-sm font-medium shadow transition-colors focus-visible:ring-1 focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50"
      >
        {isSaving ? '保存中...' : '更新资料'}
      </button>
    </div>
  {/if}
</ContentSection>
