import type {
  SqlQueryCollectionFlag,
  SqlQueryVizConfig,
} from '$lib/api/types.js'

export interface SqlQuerySnapshotFields {
  queryName: string
  queryDesc: string
  sqlContent: string
  isCollection: SqlQueryCollectionFlag
  dataSourceId: number
  module: string
  category: string
  vizConfig: SqlQueryVizConfig
}

export function buildSnapshot(fields: SqlQuerySnapshotFields): string {
  return JSON.stringify({
    queryName: fields.queryName,
    queryDesc: fields.queryDesc,
    sqlContent: fields.sqlContent,
    isCollection: fields.isCollection,
    dataSourceId: fields.dataSourceId,
    module: fields.module,
    category: fields.category,
    vizConfig: fields.vizConfig,
  })
}
