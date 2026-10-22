<script lang="ts">
  import { isApiClientNotifiedError } from '$lib/api/client.js'
  import { usersApi } from '$lib/api/users.js'
  import PasswordInput from '$lib/components/PasswordInput.svelte'
  import ContentSection from '$lib/components/settings/ContentSection.svelte'
  import { Button } from '$lib/components/ui/button/index.js'
  import * as Field from '$lib/components/ui/field/index.js'
  import { useAuth } from '$lib/context/auth.svelte.js'
  import { LoaderCircle, CheckCircle2 } from '@lucide/svelte'
  import { toast } from 'svelte-sonner'

  const auth = useAuth()

  // Form state
  let currentPassword = $state('')
  let newPassword = $state('')
  let confirmPassword = $state('')
  let isLoading = $state(false)
  let errorMessage = $state<string | null>(null)
  let successMessage = $state<string | null>(null)

  // Validation state
  let confirmError = $state<string | null>(null)

  function validatePasswords(): boolean {
    confirmError = null

    if (newPassword !== confirmPassword) {
      confirmError = '新密码和确认密码不匹配'
      return false
    }

    if (newPassword.length < 6) {
      confirmError = '新密码至少需要 6 个字符'
      return false
    }

    return true
  }

  async function handleSubmit(event: Event) {
    event.preventDefault()
    errorMessage = null
    successMessage = null

    // Client-side validation
    if (!validatePasswords()) {
      return
    }

    if (!currentPassword) {
      errorMessage = '请输入当前密码'
      return
    }

    const userId = auth.user?.id
    if (!userId) {
      errorMessage = '用户未认证'
      return
    }

    isLoading = true

    try {
      await usersApi.updatePassword(userId, newPassword)

      // Success - clear form and show message
      successMessage = '密码更新成功'
      currentPassword = ''
      newPassword = ''
      confirmPassword = ''
      toast.success('密码更新成功')
    } catch (err) {
      if (isApiClientNotifiedError(err)) {
        errorMessage = null
      } else {
        errorMessage = err instanceof Error ? err.message : '更新密码失败'
        toast.error(errorMessage)
      }
    } finally {
      isLoading = false
    }
  }
</script>

<ContentSection
  title="账户"
  desc="更新您的密码。您需要输入当前密码才能进行更改。"
>
  <form class="space-y-6" onsubmit={handleSubmit}>
    <!-- Current Password -->
    <Field.Group>
      <Field.Field>
        <Field.Label>当前密码</Field.Label>
        <PasswordInput
          placeholder="请输入当前密码"
          bind:value={currentPassword}
          disabled={isLoading}
          required
        />
      </Field.Field>
    </Field.Group>

    <!-- New Password -->
    <Field.Group>
      <Field.Field>
        <Field.Label>新密码</Field.Label>
        <PasswordInput
          placeholder="请输入新密码"
          bind:value={newPassword}
          disabled={isLoading}
          required
        />
        <Field.Description>至少 6 个字符</Field.Description>
      </Field.Field>
    </Field.Group>

    <!-- Confirm Password -->
    <Field.Group>
      <Field.Field>
        <Field.Label>确认新密码</Field.Label>
        <PasswordInput
          placeholder="请确认新密码"
          bind:value={confirmPassword}
          disabled={isLoading}
          required
        />
      </Field.Field>
      {#if confirmError}
        <Field.Error>{confirmError}</Field.Error>
      {/if}
    </Field.Group>

    <!-- Error Message -->
    {#if errorMessage}
      <div class="text-destructive text-sm">{errorMessage}</div>
    {/if}

    <!-- Success Message -->
    {#if successMessage}
      <div class="flex items-center gap-2 text-sm text-green-600">
        <CheckCircle2 class="h-4 w-4" />
        <span>{successMessage}</span>
      </div>
    {/if}

    <!-- Submit Button -->
    <Button type="submit" disabled={isLoading}>
      {#if isLoading}
        <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
        更新中...
      {:else}
        更新密码
      {/if}
    </Button>
  </form>
</ContentSection>
