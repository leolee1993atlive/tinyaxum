import { format, type FormatOptionsWithLanguage } from 'sql-formatter'

const SENTINEL_PREFIX = '__HDA_MSK_'
const SENTINEL_RE = /__HDA_MSK_(\d+)__/g

// [[..]] 可选块与 {{..}} 参数不属于 SQL 语法,先替换成单词形态的
// 占位符再交给 sql-formatter,结束后原样还原。可选块整体作为一个
// 占位符:块内 SQL(常含子查询)保持原样不重排,避免被拆成多行后
// 既难读、又超出"单行可选块"的模板约定。
const OPTIONAL_BLOCK_RE = /\[\[[\s\S]*?\]\]/g
const MUSTACHE_PLACEHOLDER_RE = /\{\{\s*[^{}]+?\s*\}\}/g

export function formatSql(
  sqlContent: string,
  options: FormatOptionsWithLanguage = {}
): string {
  if (!sqlContent.trim()) return sqlContent

  const placeholders: string[] = []

  function mask(regex: RegExp, source: string): string {
    return source.replace(regex, (match) => {
      const index = placeholders.length
      placeholders.push(match)
      return `${SENTINEL_PREFIX}${index}__`
    })
  }

  const guarded = mask(
    MUSTACHE_PLACEHOLDER_RE,
    mask(OPTIONAL_BLOCK_RE, sqlContent)
  )

  const formatted = format(guarded, {
    language: 'postgresql',
    keywordCase: 'upper',
    indentStyle: 'standard',
    linesBetweenQueries: 2,
    tabWidth: 2,
    ...options,
  })

  const restored = formatted.replace(
    SENTINEL_RE,
    (_, i) => placeholders[Number(i)] ?? ''
  )

  // 格式化器会把相邻的多个可选块并到同一行;还原后让每个块独立
  // 成行,保持"一行一个可选条件"的书写习惯。
  return restored
    .split('\n')
    .map((line) => {
      const indent = line.match(/^[ \t]*/)?.[0] ?? ''
      return line.replace(/(\S)[ \t]+(\[\[)/g, `$1\n${indent}$2`)
    })
    .join('\n')
}
