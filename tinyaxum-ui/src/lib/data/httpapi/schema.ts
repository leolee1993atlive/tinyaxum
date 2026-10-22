import { z } from 'zod'

export interface HttpApi {
  httpApiId: number
  httpApiName: string
  httpApiPath: string
  httpApiDesc?: string
  responseTemplate?: string
  enabled: number
  module?: string
  category?: string
  defaultParams?: Record<string, string | number | boolean> | null
  createdDate: string
  lastModifiedDate: string
}

export interface HttpApiQuery {
  httpApiQueryId: number
  httpApiId: number
  sqlQueryId: number
  queryAlias: string
  sortOrder: number
}

export interface HttpApiQueryDetail {
  httpApiQueryId: number
  httpApiId: number
  sqlQueryId: number
  queryAlias: string
  sortOrder: number
  sqlQueryName?: string
}

export interface HttpApiDetail extends HttpApi {
  queries: HttpApiQueryDetail[]
}

export interface HttpApiQueryInput {
  sqlQueryId: number
  queryAlias: string
  sortOrder: number
}

export const httpApiCreateSchema = z.object({
  httpApiName: z.string().min(1, 'API名称不能为空'),
  httpApiPath: z
    .string()
    .min(1, '访问路径不能为空')
    .regex(
      /^[a-zA-Z0-9_\-/]+$/,
      '路径只能包含字母、数字、下划线、中划线和斜杠'
    ),
  httpApiDesc: z.string().optional(),
  module: z.string().optional(),
  category: z.string().optional(),
  enabled: z.number().default(1),
  responseTemplate: z.string().optional(),
  defaultParams: z
    .record(z.string(), z.union([z.string(), z.number(), z.boolean()]))
    .optional(),
  queries: z
    .array(
      z.object({
        sqlQueryId: z.number().positive('请选择SQL查询'),
        queryAlias: z
          .string()
          .min(1, '别名不能为空')
          .regex(/^[a-zA-Z_][a-zA-Z0-9_]*$/, '别名格式无效'),
        sortOrder: z.number().default(0),
      })
    )
    .min(1, '至少关联一个SQL查询'),
})

export const httpApiUpdateSchema = httpApiCreateSchema.extend({
  httpApiId: z.number().positive(),
})

export type HttpApiCreateInput = z.infer<typeof httpApiCreateSchema>
export type HttpApiUpdateInput = z.infer<typeof httpApiUpdateSchema>

export const HTTP_API_COLUMNS = [
  { accessorKey: 'httpApiName', header: 'API名称' },
  { accessorKey: 'httpApiPath', header: '访问路径' },
  { accessorKey: 'module', header: '功能模块' },
  { accessorKey: 'httpApiDesc', header: '描述' },
  { accessorKey: 'enabled', header: '状态' },
  { accessorKey: 'createdDate', header: '创建时间' },
  { accessorKey: 'actions', header: '操作' },
] as const
