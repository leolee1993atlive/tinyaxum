import { z } from 'zod'

// BaseVo fields from backend
const baseEntitySchema = z.object({
  version: z.number(),
  createdBy: z.number(),
  createdDate: z.string(),
  lastModifiedBy: z.number(),
  lastModifiedDate: z.string(),
  deletedBy: z.number().optional(),
  deletedDate: z.string().optional(),
})

// TransTaskListVo from backend (matches TransTaskListVo in trans_task.rs)
export const transTaskSchema = z
  .object({
    transTaskId: z.number(),
    dataSourceId: z.number(),
    tableName: z.string(),
    tableComment: z.string(),
    remark: z.string().nullable(),
    rowCount: z.number(),
    lastTransTime: z.string(),
    taskCron: z.string(),
    taskStatus: z.string(),
    syncMode: z.string(),
    syncStrategy: z.string().nullable().optional(),
    syncColumn: z.string().nullable().optional(),
    syncWatermark: z.string().nullable().optional(),
    syncPrimaryKey: z.string().nullable().optional(),
    isSyncing: z.number(),
  })
  .merge(baseEntitySchema)

export type TransTask = z.infer<typeof transTaskSchema>

// TransTaskCreateBo for create operations
export const transTaskCreateSchema = z.object({
  dataSourceId: z.number().min(1, 'Data source is required'),
  tableName: z.string().min(1, 'Table name is required'),
  tableComment: z.string().min(1, 'Table comment is required'),
  remark: z.string().optional(),
  taskCron: z.string().min(1, 'Cron expression is required'),
  taskStatus: z.string(),
  syncMode: z.string().optional(),
  syncStrategy: z.string().optional(),
  syncColumn: z.string().optional(),
  syncPrimaryKey: z.string().optional(),
})

export type TransTaskCreate = z.infer<typeof transTaskCreateSchema>

/** 与后端批量接口上限一致 */
export const TRANS_TASK_BATCH_MAX_TABLES = 500

const transTaskBatchTableSchema = z.object({
  tableName: z.string().min(1),
  tableComment: z.string().min(1),
})

export const transTaskBatchCreateRequestSchema = z.object({
  dataSourceId: z.number().min(1, '数据源为必填项。'),
  tables: z
    .array(transTaskBatchTableSchema)
    .min(1, '请至少选择一张表。')
    .max(
      TRANS_TASK_BATCH_MAX_TABLES,
      `一次最多添加 ${TRANS_TASK_BATCH_MAX_TABLES} 张表。`
    ),
  taskCron: z.string().min(1, 'Cron表达式为必填项。'),
  remark: z.string().optional(),
  taskStatus: z.string().min(1),
  syncMode: z.string().optional(),
  syncStrategy: z.string().optional(),
  syncColumn: z.string().optional(),
  syncPrimaryKey: z.string().optional(),
})

export type TransTaskBatchCreateRequest = z.infer<
  typeof transTaskBatchCreateRequestSchema
>

// TransTaskUpdateBo for update operations
export const transTaskUpdateSchema = z.object({
  transTaskId: z.number().min(1, 'Task ID is required'),
  dataSourceId: z.number().min(1, 'Data source is required'),
  tableName: z.string().min(1, 'Table name is required'),
  tableComment: z.string().min(1, 'Table comment is required'),
  remark: z.string().optional(),
  taskCron: z.string().min(1, 'Cron expression is required'),
  taskStatus: z.string(),
  syncMode: z.string().optional(),
  syncStrategy: z.string().optional(),
  syncColumn: z.string().optional(),
  syncPrimaryKey: z.string().optional(),
})

export type TransTaskUpdate = z.infer<typeof transTaskUpdateSchema>

// Legacy Task type alias for backward compatibility
export const taskSchema = transTaskSchema
export type Task = TransTask
