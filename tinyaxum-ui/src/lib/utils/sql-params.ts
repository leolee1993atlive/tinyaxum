/**
 * SQL Parameter extraction utility
 * Mirrors backend SqlParser regex patterns
 */

// JS 的 \w 只匹配 ASCII；后端 Rust regex 的 \w 是 Unicode 感知，
// 片段名/参数名允许中文（如 {{snippet: 质控19_分子主体}}），需对齐
const IDENT = '[\\p{L}\\p{M}\\p{N}_]'
const PARAM_REGEX = new RegExp(`\\{\\{(${IDENT}+)\\}\\}`, 'gu')
// Non-global: RegExp#test on a global regex advances lastIndex and
// misjudges repeated calls
const SNIPPET_TEST_REGEX = new RegExp(
  `\\{\\{snippet:\\s*${IDENT}+\\s*\\}\\}`,
  'u'
)
const SNIPPET_NAME_REGEX = new RegExp(
  `\\{\\{snippet:\\s*(${IDENT}+)\\s*\\}\\}`,
  'gu'
)

/** 片段嵌套展开上限，与后端 SqlParser::expand_snippets 保持一致 */
const MAX_SNIPPET_DEPTH = 5

/**
 * 展开 {{snippet:name}} 引用，支持嵌套片段（上限 MAX_SNIPPET_DEPTH 层）。
 * 片段缺失或循环引用时保守保留原文，不报错。
 * 供参数提取、SQL 预览与 AI 附带 SQL 复用。
 */
export function expandSnippets(
  sql: string,
  snippets: Record<string, string>
): string {
  let result = sql
  for (let depth = 0; depth < MAX_SNIPPET_DEPTH; depth++) {
    if (!SNIPPET_TEST_REGEX.test(result)) return result
    result = result.replace(
      SNIPPET_NAME_REGEX,
      (match, name: string) => snippets[name] ?? match
    )
  }
  return result
}

/**
 * Extract snippet names referenced in SQL ({{snippet:name}}),
 * deduplicated and in order of first appearance
 */
export function extractSnippetRefs(sql: string): string[] {
  const names = new Set<string>()
  let match: RegExpExecArray | null

  SNIPPET_NAME_REGEX.lastIndex = 0

  while ((match = SNIPPET_NAME_REGEX.exec(sql)) !== null) {
    names.add(match[1])
  }

  return Array.from(names)
}

/**
 * Extract parameter names from SQL content
 * Parses {{param_name}} patterns, excluding {{snippet:...}}
 *
 * @param sql - SQL query content
 * @param snippets - Optional snippet name→content map; when provided,
 *   snippet references are expanded first so params that only exist
 *   inside snippet content are discovered (mirrors backend behavior)
 * @returns Array of unique parameter names
 */
export function extractParams(
  sql: string,
  snippets?: Record<string, string>
): string[] {
  const expanded = snippets ? expandSnippets(sql, snippets) : sql
  const params = new Set<string>()
  let match: RegExpExecArray | null

  // Reset regex lastIndex
  PARAM_REGEX.lastIndex = 0

  while ((match = PARAM_REGEX.exec(expanded)) !== null) {
    // {{snippet:x}} contains a colon and never matches PARAM_REGEX
    params.add(match[1])
  }

  return Array.from(params)
}

/**
 * Check if a string is a snippet reference
 *
 * @param text - Text to check
 * @returns true if it's a snippet reference
 */
export function isSnippetReference(text: string): boolean {
  return SNIPPET_TEST_REGEX.test(text)
}

/**
 * Create snippet insertion text
 *
 * @param snippetName - Name of the snippet
 * @returns Formatted snippet reference
 */
export function createSnippetReference(snippetName: string): string {
  return `{{snippet: ${snippetName}}}`
}
