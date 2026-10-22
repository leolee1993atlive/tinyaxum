// API Response Types - matches backend R<T> structure
export interface ApiResponse<T> {
  code: number
  message: string
  data?: T
}

export interface ApiError {
  code: string
  message: string
  details?: Record<string, string[]>
}

// Pagination Types
export interface PaginationParams {
  pageNo: number
  pageSize: number
  sort?: string
  order?: 'asc' | 'desc'
}

/** Matches backend `PageResult` (common/vo/page.rs), camelCase. */
export interface PageResult<T> {
  records: T[]
  total: number
  pages: number
  current: number
  size: number
}

export type UserPageParams = PaginationParams & {
  keyword?: string
  username?: string
  nickName?: string
  active?: string
}

export type DatasourcePageParams = PaginationParams & {
  keyword?: string
  code?: string
  name?: string
  dbType?: string
}

export type TransTaskPageParams = PaginationParams & {
  keyword?: string
  dataSourceId?: number
  tableName?: string
  tableComment?: string
  taskStatus?: string
}

export type SqlQueryPageParams = PaginationParams & {
  keyword?: string
  sqlQueryName?: string
  dataSourceId?: number
  module?: string
  category?: string
}

export type SqlSnippetPageParams = PaginationParams & {
  keyword?: string
  sqlSnippetName?: string
  tag?: string
}

/**
 * Serialize pagination + optional filters for GET query string.
 * Omits undefined/null and empty-string optional filters.
 */
export function toPageQueryRecord(
  base: PaginationParams,
  extra?: Record<string, string | number | undefined | null>
): Record<string, string> {
  const r: Record<string, string> = {
    pageNo: String(base.pageNo),
    pageSize: String(base.pageSize),
  }
  if (base.sort) r.sort = base.sort
  if (base.order) r.order = base.order
  if (!extra) return r
  for (const [k, v] of Object.entries(extra)) {
    if (v === undefined || v === null) continue
    if (typeof v === 'string' && v.trim() === '') continue
    r[k] = String(v)
  }
  return r
}

// User API Types - matches backend UserCreateBo, UserUpdateBo
export interface UserCreateRequest {
  username: string
  password: string
  nickName: string
  email?: string
  active: string
}

export interface UserUpdateRequest {
  userId: number
  username: string
  nickName: string
  email?: string
  active: string
}

// Auth API Types - matches backend AuthBody and AuthPayload
export interface LoginRequest {
  username: string
  password: string
}

export interface LoginResponse {
  accessToken: string
  refreshToken: string
  tokenType: string
}

export interface RegisterRequest {
  firstName: string
  lastName: string
  email: string
  password: string
}

export interface ForgotPasswordRequest {
  email: string
}

// Datasource API Types - matches backend DataSourceCreateBo, DataSourceUpdateBo, DataSourceListVo, DataSourceDetailVo
export interface DatasourceCreateRequest {
  code: string
  name: string
  remark?: string
  dbType: string
  dbHost: string
  dbPort: number
  dbName: string
  dbSchema?: string
  dbUsername: string
  dbPassword: string
}

export interface DatasourceUpdateRequest extends DatasourceCreateRequest {
  dataSourceId: number
}

export interface DatasourceListVo {
  dataSourceId: number
  code: string
  name: string
  remark?: string
  dbType: string
  dbName: string
  createdBy: number
  createdDate: string
  lastModifiedBy: number
  lastModifiedDate: string
}

export interface DatasourceDetailVo {
  dataSourceId: number
  code: string
  name: string
  remark?: string
  dbType: string
  dbHost: string
  dbPort: number
  dbName: string
  dbSchema?: string
  dbUsername: string
  createdBy: number
  createdDate: string
  lastModifiedBy: number
  lastModifiedDate: string
}

export interface DatasourceTestResult {
  success: boolean
  message: string
}

/** GET /datasource/{id}/schema */
export interface DatasourceColumnSchemaVo {
  columnMetaId: number
  columnName: string
  commentLabel?: string
  fieldType: string
  formatPattern?: string
  fieldDesc?: string
  sourcePhysicalType?: string
  ordinalPosition: number
}

export interface DatasourceTableSchemaVo {
  tableMetaId: number
  tableName: string
  commentLabel?: string
  tableDesc?: string
  columns: DatasourceColumnSchemaVo[]
}

export interface DatasourceSchemaVo {
  dataSourceId: number
  tables: DatasourceTableSchemaVo[]
}

// SQL Query — visualization config (persisted JSON, version 1)
export type SqlQueryVizColumnAlign = 'left' | 'center' | 'right'

export interface SqlQueryVizTableColumn {
  fieldName: string
  title?: string
  align: SqlQueryVizColumnAlign
  visible: boolean
}

export type SqlQueryVizChartSeriesType = 'line' | 'area'

export interface SqlQueryVizChartSeries {
  fieldName: string
  type: SqlQueryVizChartSeriesType
  label?: string
}

export interface SqlQueryVizChart {
  xField: string
  series: SqlQueryVizChartSeries[]
}

export interface SqlQueryVizConfig {
  version: 1
  scalarField?: string
  kind?: 'table' | 'chart'
  table?: {
    showRowIndex: boolean
    columns: SqlQueryVizTableColumn[]
  }
  chart?: SqlQueryVizChart
}

/** 0 单行/标量，1 多条（集合），与后端 i8 / 库中整数一致 */
export type SqlQueryCollectionFlag = 0 | 1

// SQL Query API Types - matches backend SqlQueryCreateBo, SqlQueryUpdateBo, SqlQueryListVo, SqlQueryDetailVo
export interface SqlQueryCreateRequest {
  sqlQueryName: string
  sqlQueryDesc?: string
  sqlQueryContent: string
  dataSourceId: number
  /** 缺省由后端视为 1（集合） */
  isCollection?: SqlQueryCollectionFlag
  vizConfig?: SqlQueryVizConfig | null
  module?: string
  category?: string
}

export interface SqlQueryUpdateRequest {
  sqlQueryId: number
  sqlQueryName: string
  sqlQueryDesc?: string
  sqlQueryContent: string
  dataSourceId: number
  isCollection: SqlQueryCollectionFlag
  vizConfig?: SqlQueryVizConfig | null
  module?: string
  category?: string
}

/** 对应后端 `SqlQueryVizConfigSaveBo`，仅保存可视化 JSON */
export interface SqlQueryVizConfigSaveRequest {
  vizConfig?: SqlQueryVizConfig | null
}

/** 列表/分页（不含 SQL 正文，减小 payload） */
export interface SqlQueryListVo {
  sqlQueryId: number
  sqlQueryName: string
  sqlQueryDesc?: string
  dataSourceId: number
  isCollection: SqlQueryCollectionFlag
  vizKind?: 'scalar' | 'table' | 'chart' | null
  module?: string | null
  category?: string | null
  createdBy: number
  createdDate: string
  lastModifiedBy: number
  lastModifiedDate: string
}

export interface SqlQueryDetailVo {
  sqlQueryId: number
  sqlQueryName: string
  sqlQueryDesc?: string
  sqlQueryContent: string
  dataSourceId: number
  isCollection: SqlQueryCollectionFlag
  resultMetaUpdatedAt?: string | null
  vizConfig?: SqlQueryVizConfig | null
  module?: string
  category?: string
  resultFields?: SqlQueryResultFieldVo[]
  paramFields?: SqlQueryParamFieldVo[]
  createdBy: number
  createdDate: string
  lastModifiedBy: number
  lastModifiedDate: string
}

/** 与后端 BaseVo 扁平字段一致 */
export interface SqlFieldMetaBase {
  version: number
  createdBy: number
  createdDate: string
  lastModifiedBy: number
  lastModifiedDate: string
  deletedBy?: number
  deletedDate?: string
}

export interface SqlQueryResultFieldVo extends SqlFieldMetaBase {
  resultFieldId: number
  fieldName: string
  commentLabel?: string
  fieldType: string
  formatPattern?: string
  fieldDesc?: string
}

export interface SqlQueryParamFieldVo extends SqlFieldMetaBase {
  paramFieldId: number
  fieldName: string
  commentLabel?: string
  fieldType: string
  formatPattern?: string
  fieldDesc?: string
  required: boolean
}

export type QueryResultRow = Record<string, unknown>

/** 运行 SQL 返回：列物理类型 + 行数据 */
export interface SqlResultColumnVo {
  name: string
  physicalType: string
}

export interface SqlRunResultVo {
  columns: SqlResultColumnVo[]
  rows: QueryResultRow[]
  total?: number
  current?: number
  size?: number
  pages?: number
}

export interface DashboardRunVo {
  result: SqlRunResultVo
  cached: boolean
}

// Dashboard config (persisted JSON, version 1)
export type DashboardConfigItem = {
  sqlQueryId: number
  defaults?: Record<string, string>
}

export interface DashboardConfigV1 {
  version: 1
  tabs: {
    overview: { items: DashboardConfigItem[] }
    analytics: { items: DashboardConfigItem[] }
    charts: { items: DashboardConfigItem[] }
  }
}

/** 对应后端 `DashboardConfigVo` */
export interface DashboardConfigVo {
  config: DashboardConfigV1
}

/** 对应后端 `DashboardConfigSaveBo` */
export interface DashboardConfigSaveRequest {
  config: DashboardConfigV1
}

// Ad-hoc SQL Query Execution Request
export interface SqlQueryRunAdhocRequest {
  sqlQueryContent: string
  dataSourceId?: number
  params?: Record<string, string>
  pageNo?: number
  pageSize?: number
}

// SQL Snippet API Types - matches backend SqlSnippetCreateBo, SqlSnippetUpdateBo, SqlSnippetListVo
export interface SqlSnippetCreateRequest {
  sqlSnippetName: string
  sqlSnippetDesc?: string
  sqlSnippetContent: string
  tags?: string
}

export interface SqlSnippetUpdateRequest {
  sqlSnippetId: number
  sqlSnippetName: string
  sqlSnippetDesc?: string
  sqlSnippetContent: string
  tags?: string
}

export interface SqlSnippetListVo {
  sqlSnippetId: number
  sqlSnippetName: string
  sqlSnippetDesc?: string
  sqlSnippetContent: string
  tags?: string
  createdBy: number
  createdDate: string
  lastModifiedBy: number
  lastModifiedDate: string
}

// ============ AI Provider ============
export interface AiProviderListVo {
  aiProviderId: number
  name: string
  baseUrl: string
  apiKeyMasked: string
  model: string
  systemPrompt: string
  temperature: number
  isDefault: number
  version: number
  createdBy: number
  createdDate: string
  lastModifiedBy: number
  lastModifiedDate: string
  deletedBy: number | null
  deletedDate: string | null
}

export interface AiProviderCreateRequest {
  name: string
  baseUrl: string
  apiKey: string
  model: string
  systemPrompt: string
  temperature?: number
}

export interface AiProviderUpdateRequest {
  name: string
  baseUrl: string
  apiKey: string
  model: string
  systemPrompt: string
  temperature?: number
}

export interface AiProviderTestResultVo {
  ok: boolean
  latencyMs: number
  modelEcho: string | null
  error: string | null
}

// ============ AI Chat ============
export interface AiConversationListVo {
  aiConversationId: number
  sqlQueryId: number
  aiProviderId: number
  providerName: string | null
  title: string
  version: number
  createdBy: number
  createdDate: string
  lastModifiedBy: number
  lastModifiedDate: string
  deletedBy: number | null
  deletedDate: string | null
}

export interface AiConversationCreateRequest {
  sqlQueryId: number
  aiProviderId?: number
  title?: string
}

export interface AiConversationUpdateRequest {
  title?: string
  aiProviderId?: number
}

export interface AiMessageVo {
  aiMessageId: number
  aiConversationId: number
  role: 'user' | 'assistant'
  content: string
  attachedSchema: number
  promptTokens: number | null
  completionTokens: number | null
  version: number
  createdBy: number
  createdDate: string
  lastModifiedBy: number
  lastModifiedDate: string
  deletedBy: number | null
  deletedDate: string | null
}

export interface SendMessageRequest {
  content: string
  attachSchema: boolean
  currentSql?: string
}

export interface ModuleOptionVo {
  name: string
  categories: string[]
}

export type AiSseEvent =
  | { type: 'user_saved'; id: number }
  | { type: 'delta'; content: string }
  | {
      type: 'done'
      id: number
      promptTokens: number | null
      completionTokens: number | null
    }
  | { type: 'error'; code: string; message: string }
