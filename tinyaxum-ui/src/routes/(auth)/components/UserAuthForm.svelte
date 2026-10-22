<script lang="ts">
  import { goto } from '$app/navigation'
  import PasswordInput from '$lib/components/PasswordInput.svelte'
  import { Button } from '$lib/components/ui/button/index.js'
  import * as Field from '$lib/components/ui/field/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import { useAuth } from '$lib/context/auth.svelte.js'
  import { cn } from '$lib/utils.js'
  import { LoaderCircle, LogIn } from '@lucide/svelte'
  import { toast } from 'svelte-sonner'

  let { class: className = '', redirectTo = '/users' } = $props()

  const auth = useAuth()

  let username = $state('')
  let password = $state('')

  async function handleSubmit(event: Event) {
    event.preventDefault()

    if (!username || !password) {
      toast.error('请输入用户名和密码')
      return
    }

    const success = await auth.login(username, password)

    if (success) {
      toast.success('登录成功！')
      await goto(redirectTo)
    }
  }
</script>

<form class={cn('grid gap-3', className)} onsubmit={handleSubmit}>
  <Field.Group>
    <Field.Field>
      <Field.Label>用户名</Field.Label>
      <Input bind:value={username} required disabled={auth.isLoading} />
    </Field.Field>
  </Field.Group>
  <Field.Group>
    <Field.Field class="relative">
      <div class="flex items-center">
        <Field.Label>密码</Field.Label>
      </div>
      <PasswordInput bind:value={password} required disabled={auth.isLoading} />
    </Field.Field>
  </Field.Group>
  {#if auth.error}
    <div class="text-destructive text-sm">{auth.error}</div>
  {/if}
  <Button class="mt-2" type="submit" disabled={auth.isLoading}>
    {#if auth.isLoading}
      <LoaderCircle class="animate-spin" />
    {:else}
      <LogIn />
    {/if}
    登录
  </Button>
</form>
