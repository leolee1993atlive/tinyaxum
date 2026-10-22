import type { DatasourceTableSchemaVo } from '$lib/api/types.js'

export interface CmSchema {
  [tableName: string]: string[]
}

export function buildCmSchema(tables: DatasourceTableSchemaVo[]): CmSchema {
  const schema: CmSchema = {}
  for (const table of tables) {
    schema[table.tableName] = table.columns.map((c) => c.columnName)
  }
  return schema
}
