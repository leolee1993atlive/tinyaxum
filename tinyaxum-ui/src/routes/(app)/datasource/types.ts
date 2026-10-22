export interface Datasource {
  dataSourceId: number
  code: string
  name: string
  remark?: string
  dbType: string
  dbName: string
  createdBy?: string
  createdDate?: string
  updatedBy?: string
  updatedDate?: string
}
