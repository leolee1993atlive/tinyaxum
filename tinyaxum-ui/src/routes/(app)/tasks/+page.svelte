<script lang="ts">
  import ProfileDropdown from '$lib/components/ProfileDropdown.svelte'
  import Search from '$lib/components/Search.svelte'
  import Header from '$lib/components/layout/Header.svelte'
  import Main from '$lib/components/layout/Main.svelte'
  import TasksDialogs from './components/TasksDialogs.svelte'
  import TasksPrimaryButtons from './components/TasksPrimaryButtons.svelte'
  import TasksProvider from './components/TasksProvider.svelte'
  import TasksTable from './components/TasksTable.svelte'
</script>

<TasksProvider>
  <Header>
    <Search />
    <ProfileDropdown />
  </Header>

  <Main class="flex flex-1 flex-col gap-4 sm:gap-6">
    <div class="flex flex-wrap items-end justify-between gap-2">
      <div>
        <h2 class="text-2xl font-bold tracking-tight">传输任务</h2>
        <p class="text-muted-foreground">在此处管理您的定时数据传输任务。</p>
      </div>
      <TasksPrimaryButtons />
    </div>
    <div class="-mx-4 flex-1 overflow-auto px-4 py-1 lg:mx-0 lg:px-0">
      <TasksTable />
    </div>
  </Main>

  <TasksDialogs />
</TasksProvider>
