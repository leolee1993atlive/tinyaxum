<script lang="ts">
  import { untrack } from 'svelte'
  import { isApiClientNotifiedError } from '$lib/api/client.js'
  import { usersApi } from '$lib/api/users.js'
  import { Button } from '$lib/components/ui/button/index.js'
  import * as Field from '$lib/components/ui/field/index.js'
  import { Input } from '$lib/components/ui/input/index.js'
  import * as Sheet from '$lib/components/ui/sheet/index.js'
  import type {
    User,
    UserCreateInput,
    UserUpdateInput,
  } from '$lib/data/users/schema.js'
  import { LoaderCircle } from '@lucide/svelte'
  import { toast } from 'svelte-sonner'
  import { z } from 'zod'
  import { useUsers } from './users-state.svelte'

  const usersState = useUsers()

  const createSchema = z.object({
    username: z.string().min(1, '用户名为必填项。'),
    password: z.string().min(1, '密码为必填项。'),
    nickName: z.string().min(1, '昵称为必填项。'),
    email: z.string().email('邮箱格式不正确。').optional().or(z.literal('')),
    active: z.enum(['1', '0']),
  })

  const updateSchema = z.object({
    username: z.string().min(1, '用户名为必填项。'),
    nickName: z.string().min(1, '昵称为必填项。'),
    email: z.string().email('邮箱格式不正确。').optional().or(z.literal('')),
    active: z.enum(['1', '0']),
  })

  type CreateForm = z.infer<typeof createSchema>
  type UpdateForm = z.infer<typeof updateSchema>

  let formData = $state<CreateForm>({
    username: '',
    password: '',
    nickName: '',
    email: '',
    active: '1',
  })

  let errors = $state<Record<string, string[]>>({})
  let isSubmitting = $state(false)

  const isUpdate = $derived(usersState.open === 'update')

  function resetForm() {
    formData = {
      username: '',
      password: '',
      nickName: '',
      email: '',
      active: '1',
    }
    errors = {}
  }

  $effect(() => {
    if (usersState.open === 'update' && usersState.currentRow) {
      const user = usersState.currentRow as User
      untrack(() => {
        formData = {
          username: user.username,
          password: '',
          nickName: user.nickName,
          email: user.email || '',
          active: user.active as '0' | '1',
        }
      })
    } else if (usersState.open === 'create') {
      untrack(() => resetForm())
    }
    errors = {}
  })

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault()

    const schemaToUse = isUpdate ? updateSchema : createSchema
    const parseResult = schemaToUse.safeParse(formData)

    if (!parseResult.success) {
      errors = parseResult.error.flatten().fieldErrors as Record<
        string,
        string[]
      >
      return
    }

    errors = {}
    isSubmitting = true

    try {
      if (isUpdate && usersState.currentRow) {
        const user = usersState.currentRow as User
        const input: UserUpdateInput = {
          userId: user.userId,
          username: parseResult.data.username,
          nickName: parseResult.data.nickName,
          email: parseResult.data.email || undefined,
          active: parseResult.data.active,
        }
        await usersApi.updateUser(input)
        toast.success('用户更新成功')
      } else {
        const parsedFormData = parseResult.data as CreateForm
        const input: UserCreateInput = {
          username: parsedFormData.username,
          password: parsedFormData.password,
          nickName: parsedFormData.nickName,
          email: parsedFormData.email || undefined,
          active: parsedFormData.active,
        }
        await usersApi.createUser(input)
        toast.success('用户创建成功')
      }
      usersState.setOpen(null)
      usersState.requestListRefetch()
    } catch (e) {
      const message =
        e instanceof Error
          ? e.message
          : isUpdate
            ? '更新用户失败'
            : '创建用户失败'
      if (!isApiClientNotifiedError(e)) {
        errors = { _form: [message] }
        toast.error(message)
      }
    } finally {
      isSubmitting = false
    }
  }

  function handleOpenChange(open: boolean) {
    if (!open) {
      usersState.setOpen(null)
      setTimeout(() => {
        usersState.setCurrentRow(null)
      }, 500)
    }
  }
</script>

<Sheet.Root
  open={usersState.open === 'create' || usersState.open === 'update'}
  onOpenChange={handleOpenChange}
>
  <Sheet.Content class="flex flex-col">
    <Sheet.Header class="text-start">
      <Sheet.Title>{isUpdate ? '编辑用户' : '创建用户'}</Sheet.Title>
      <Sheet.Description>
        {isUpdate ? '更新用户信息。' : '添加新用户到系统。'}
      </Sheet.Description>
    </Sheet.Header>

    <form
      id="user-form"
      onsubmit={handleSubmit}
      class="flex-1 space-y-4 overflow-y-auto px-4 py-4"
    >
      <Field.Field>
        <Field.Label>用户名</Field.Label>
        <Input
          bind:value={formData.username}
          placeholder="请输入用户名"
          disabled={isSubmitting}
        />
        <Field.Error
          errors={errors.username?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      {#if !isUpdate}
        <Field.Field>
          <Field.Label>密码</Field.Label>
          <Input
            type="password"
            bind:value={formData.password}
            placeholder="请输入密码"
            disabled={isSubmitting}
          />
          <Field.Error
            errors={errors.password?.map((m) => ({ message: m })) ?? []}
          />
        </Field.Field>
      {/if}

      <Field.Field>
        <Field.Label>昵称</Field.Label>
        <Input
          bind:value={formData.nickName}
          placeholder="请输入昵称"
          disabled={isSubmitting}
        />
        <Field.Error
          errors={errors.nickName?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>邮箱</Field.Label>
        <Input
          type="email"
          bind:value={formData.email}
          placeholder="请输入邮箱（可选）"
          disabled={isSubmitting}
        />
        <Field.Error
          errors={errors.email?.map((m) => ({ message: m })) ?? []}
        />
      </Field.Field>

      <Field.Field>
        <Field.Label>状态</Field.Label>
        <select
          bind:value={formData.active}
          class="border-input bg-background ring-offset-background focus-visible:ring-ring flex h-10 w-full rounded-md border px-3 py-2 text-sm focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
          disabled={isSubmitting}
        >
          <option value="1">活跃</option>
          <option value="0">停用</option>
        </select>
      </Field.Field>

      {#if errors._form}
        <div class="text-destructive text-sm">
          {errors._form.join(', ')}
        </div>
      {/if}
    </form>

    <Sheet.Footer class="flex flex-row justify-end gap-2">
      <Sheet.Close>
        <Button variant="outline" disabled={isSubmitting}>取消</Button>
      </Sheet.Close>
      <Button form="user-form" type="submit" disabled={isSubmitting}>
        {#if isSubmitting}
          <LoaderCircle class="mr-2 h-4 w-4 animate-spin" />
        {/if}
        {isUpdate ? '保存' : '创建'}
      </Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>
