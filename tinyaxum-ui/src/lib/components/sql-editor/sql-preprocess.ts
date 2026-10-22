export function preprocessSqlForParsing(sql: string): string {
  let result = sql
  result = result.replace(/\{\{snippet:\s*[^}]*\}\}/g, '')
  result = result.replace(/\{\{[^}]+\}\}/g, "''")
  result = result.replace(/\[\[/g, '')
  result = result.replace(/\]\]/g, '')
  return result
}
