<script lang="ts">
  import { goto } from '$app/navigation'
  import Logo from '$lib/components/Logo.svelte'
  import { useAuth } from '$lib/context/auth.svelte.js'

  let { children } = $props()

  const auth = useAuth()
  $effect(() => {
    if (auth.isAuthenticated) goto('/users')
  })
</script>

<svelte:head>
  <title>登录 - TinyAxum</title>
</svelte:head>

<div class="container grid h-svh max-w-none items-center justify-center">
  <div
    class="mx-auto flex w-full flex-col justify-center space-y-2 py-8 sm:w-[480px] sm:p-8"
  >
    <div class="mb-4 flex items-center justify-center">
      <Logo class="me-2" />
      <h1 class="text-xl font-medium">TinyAxum</h1>
    </div>
    {@render children()}
  </div>
</div>
