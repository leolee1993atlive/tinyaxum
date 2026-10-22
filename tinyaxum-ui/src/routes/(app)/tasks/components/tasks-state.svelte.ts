import { SvelteMap, SvelteSet } from 'svelte/reactivity'
import { isApiClientNotifiedError } from '$lib/api/client.js'
import { datasourcesApi } from '$lib/api/datasources.js'
import { transtaskApi } from '$lib/api/transtask.js'
import type { SyncProgressEvent } from '$lib/api/transtask.js'
import type { DatasourceListVo } from '$lib/api/types.js'
import {
  CrudPageState,
  createCrudContext,
} from '$lib/composables/crud-page-state.svelte.js'
import type { TransTask } from '../data/schema.js'

export type TasksDialogType =
  | 'create'
  | 'update'
  | 'delete'
  | 'batchCreate'
  | 'syncProgress'
  | 'copy'

export type TransTaskListQuery = {
  pageNo: number
  pageSize: number
  keyword?: string
  sort?: string
  order?: 'asc' | 'desc'
  dataSourceId?: number
}

export type SyncPhase = 'exporting' | 'done' | 'error'

export type TaskSyncProgress = {
  phase: SyncPhase
  rowsDone: number | null
  doneRows: number | null
  newWatermark: string | null
  errorMessage: string | null
  elapsedSeconds: number
}

export class TasksState extends CrudPageState<TransTask, TasksDialogType> {
  tasks = $state<TransTask[]>([])
  totalRowCount = $state(0)
  listQuery = $state<TransTaskListQuery>({ pageNo: 1, pageSize: 10 })
  isLoading = $state(false)
  error = $state<string | null>(null)
  /** 递增后由 TasksTable 内 useCrudTable.reload() 消费，用于列表数据与 TanStack 缓存同步 */
  listRefetchTick = $state(0)
  /** 当前正在通过 SSE 执行同步的任务 ID 集合（包含后台运行的） */
  runningTaskIds = new SvelteSet<number>()
  /** 每个任务的实时同步进度，key 为 transTaskId */
  taskProgress = new SvelteMap<number, TaskSyncProgress>()
  /** 当前正在复制的任务 ID，用于按钮 loading 状态 */
  copyingId = $state<number | null>(null)
  /** 页面共享的数据源下拉列表，table 列与 drawer 下拉共用 */
  datasources = $state<DatasourceListVo[]>([])

  private unsubscribeEvents: (() => void) | null = null
  private controllers = new Map<number, AbortController>()
  private timerHandles = new Map<number, ReturnType<typeof setInterval>>()
  private datasourcesPromise: Promise<void> | null = null

  setError = (error: string | null) => {
    this.error = error
  }

  requestListRefetch() {
    this.listRefetchTick += 1
  }

  markTaskRunning(id: number) {
    this.runningTaskIds.add(id)
  }

  markTaskDone(id: number) {
    this.runningTaskIds.delete(id)
    this.requestListRefetch()
  }

  getProgress(taskId: number): TaskSyncProgress | null {
    return this.taskProgress.get(taskId) ?? null
  }

  private patchProgress(taskId: number, patch: Partial<TaskSyncProgress>) {
    const cur = this.taskProgress.get(taskId)
    if (cur) this.taskProgress.set(taskId, { ...cur, ...patch })
  }

  private applyProgressEvent(taskId: number, event: SyncProgressEvent) {
    switch (event.type) {
      case 'exporting':
        this.patchProgress(taskId, { phase: 'exporting' })
        break
      case 'progress':
        this.patchProgress(taskId, {
          phase: 'exporting',
          rowsDone: event.rowsDone,
        })
        break
      case 'done':
        this.patchProgress(taskId, {
          phase: 'done',
          doneRows: event.rowCount,
          newWatermark: event.newWatermark,
        })
        break
      case 'error':
        this.patchProgress(taskId, {
          phase: 'error',
          errorMessage: event.message,
        })
        break
    }
  }

  /**
   * 启动指定任务的 SSE 同步流。若该任务已在运行则直接返回。
   * 进度存入 taskProgress，完成后自动清理并刷新列表。
   */
  async startTaskSync(taskId: number) {
    if (this.controllers.has(taskId)) return

    const controller = new AbortController()
    this.controllers.set(taskId, controller)

    this.taskProgress.set(taskId, {
      phase: 'exporting',
      rowsDone: null,
      doneRows: null,
      newWatermark: null,
      errorMessage: null,
      elapsedSeconds: 0,
    })
    this.markTaskRunning(taskId)

    const timer = setInterval(() => {
      const p = this.taskProgress.get(taskId)
      if (p && p.phase !== 'done' && p.phase !== 'error') {
        this.patchProgress(taskId, { elapsedSeconds: p.elapsedSeconds + 1 })
      }
    }, 1000)
    this.timerHandles.set(taskId, timer)

    try {
      for await (const event of transtaskApi.runWithProgress(
        taskId,
        controller.signal
      )) {
        this.applyProgressEvent(taskId, event)
        if (event.type === 'done' || event.type === 'error') break
      }
    } catch (err) {
      if (!(err instanceof DOMException && err.name === 'AbortError')) {
        this.patchProgress(taskId, {
          phase: 'error',
          errorMessage: err instanceof Error ? err.message : '连接失败',
        })
      }
    } finally {
      clearInterval(timer)
      this.timerHandles.delete(taskId)
      this.controllers.delete(taskId)
      this.markTaskDone(taskId)
    }
  }

  /** 中止指定任务的 SSE 同步（前端取消，后端仍会完成） */
  cancelTaskSync(taskId: number) {
    this.controllers.get(taskId)?.abort()
  }

  startEventStream() {
    if (this.unsubscribeEvents) return
    this.unsubscribeEvents = transtaskApi.subscribeToEvents(
      (event) => {
        const idx = this.tasks.findIndex((t) => t.transTaskId === event.taskId)
        if (idx >= 0) {
          this.tasks[idx] = {
            ...this.tasks[idx],
            rowCount: event.rowCount,
            lastTransTime: new Date().toISOString(),
            isSyncing: 0,
          }
        } else {
          this.requestListRefetch()
        }
      },
      () => {
        this.unsubscribeEvents = null
        setTimeout(() => this.startEventStream(), 3000)
      }
    )
  }

  stopEventStream() {
    this.unsubscribeEvents?.()
    this.unsubscribeEvents = null
  }

  /**
   * 加载数据源下拉数据，table 与 drawer 共用。重复调用会复用同一 in-flight Promise，
   * 加载成功后不会再请求；如失败则下次调用重试。
   */
  async loadDatasources(): Promise<void> {
    if (this.datasources.length > 0) return
    if (this.datasourcesPromise) return this.datasourcesPromise
    this.datasourcesPromise = (async () => {
      try {
        const res = await datasourcesApi.page({ pageNo: 1, pageSize: 200 })
        this.datasources = res.data?.records ?? []
      } catch (e) {
        this.datasourcesPromise = null
        throw e
      }
    })()
    return this.datasourcesPromise
  }

  async loadTasks(override?: Partial<TransTaskListQuery>) {
    this.isLoading = true
    this.error = null
    if (override) {
      this.listQuery = { ...this.listQuery, ...override }
    }
    const q = this.listQuery
    try {
      const res = await transtaskApi.page({
        pageNo: q.pageNo,
        pageSize: q.pageSize,
        keyword: q.keyword,
        sort: q.sort,
        order: q.order,
        dataSourceId: q.dataSourceId,
      })
      this.tasks = res.data?.records ?? []
      this.totalRowCount = Number(res.data?.total ?? 0)
    } catch (error) {
      this.error = isApiClientNotifiedError(error)
        ? null
        : error instanceof Error
          ? error.message
          : 'Failed to load tasks'
    } finally {
      this.isLoading = false
    }
  }

  async deleteCurrentTask() {
    if (!this.currentRow) return false

    try {
      const success = await transtaskApi.delete(this.currentRow.transTaskId)
      if (success) {
        this.requestListRefetch()
      }
      return success
    } catch (error) {
      this.error = isApiClientNotifiedError(error)
        ? null
        : error instanceof Error
          ? error.message
          : 'Failed to delete task'
      return false
    }
  }

  async copyTask(id: number) {
    this.copyingId = id
    this.error = null

    try {
      const task = await transtaskApi.getById(id)
      if (!task) {
        this.error = '任务不存在或已被删除'
        return false
      }

      this.currentRow = {
        ...task,
        tableName: task.tableName + '_副本',
        taskStatus: 'stop',
      }
      this.setOpen('copy')
      return true
    } catch (error) {
      this.error = isApiClientNotifiedError(error)
        ? null
        : error instanceof Error
          ? error.message
          : '复制任务失败'
      return false
    } finally {
      this.copyingId = null
    }
  }
}

const { provide: provideTasks, use: useTasks } =
  createCrudContext<TasksState>('Tasks')

export { provideTasks, useTasks }
