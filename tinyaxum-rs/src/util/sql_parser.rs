use std::collections::{HashMap, HashSet};

use regex::{Captures, Regex};

use crate::error::error::{AppError, AppResult};

pub struct SqlParser {
    parameter_regex: Regex,
    optional_regex: Regex,
    snippet_regex: Regex,
    where_leading_and_or_regex: Regex,
    where_empty_before_clause_regex: Regex,
    where_empty_before_terminator_regex: Regex,
    where_empty_at_end_regex: Regex,
}

impl SqlParser {
    pub fn new() -> AppResult<Self> {
        Ok(SqlParser {
            parameter_regex: Regex::new(r"\{\{(\w+)\}\}")?,
            // (?s):可选块允许跨行——前端 SQL 格式化会把含子查询的
            // [[AND ... (...多行...)]] 拆成多行,这里必须能整块匹配。
            optional_regex: Regex::new(r"(?s)\[\[(.*?)\]\]")?,
            snippet_regex: Regex::new(r"\{\{snippet:\s*(\w+)\s*\}\}")?,
            where_leading_and_or_regex: Regex::new(
                r"(?i)\bWHERE\s+(?:AND|OR)\b",
            )?,
            where_empty_before_clause_regex: Regex::new(
                r"(?i)\bWHERE\s+(GROUP|ORDER|LIMIT|HAVING|UNION|INTERSECT|EXCEPT|MINUS|OFFSET|FETCH|FOR|RETURNING|WINDOW)\b",
            )?,
            where_empty_before_terminator_regex: Regex::new(
                r"(?i)\bWHERE\s*([);])",
            )?,
            where_empty_at_end_regex: Regex::new(r"(?i)\bWHERE\s*\z")?,
        })
    }

    pub fn parse(
        &self,
        sql_template: &str,
        parameters: &Option<HashMap<String, String>>,
        snippets: &Option<HashMap<String, String>>,
    ) -> AppResult<String> {
        let mut result = sql_template.to_string();

        // 1. 先展开代码片段 {{snippet: ...}}：片段内容中可以继续使用
        //    [[...]] 可选块与 {{param}} 参数，交由后续步骤统一求值
        result = self.expand_snippets(&result, snippets)?;

        // 2. 处理可选参数 [[...]]
        result = self.process_optional_blocks(&result, parameters)?;

        // 3. 处理普通参数 {{...}}
        result = self.process_parameters(&result, parameters)?;

        // 4. 清理 WHERE 残留（首个 AND/OR、空 WHERE 子句）
        result = self.cleanup_where_clauses(&result);

        Ok(result)
    }

    /// 展开代码片段引用。循环展开以支持嵌套片段，层数上限防止
    /// 循环引用（如片段内容引用自身）导致无限膨胀。
    pub fn expand_snippets(
        &self,
        sql: &str,
        snippets: &Option<HashMap<String, String>>,
    ) -> AppResult<String> {
        const MAX_SNIPPET_DEPTH: usize = 5;

        let mut result = sql.to_string();
        let mut depth = 0usize;
        while self.snippet_regex.is_match(&result) {
            depth += 1;
            if depth > MAX_SNIPPET_DEPTH {
                return Err(AppError::BadRequest(format!(
                    "代码片段嵌套超过 {MAX_SNIPPET_DEPTH} 层或存在循环引用"
                )));
            }
            result = self.process_snippets(&result, snippets)?;
        }
        Ok(result)
    }

    /// 模仿 MyBatis `<where>` 标签的清理：当所有 `[[AND ...]]` 都被剔除后，
    /// 自动去掉孤立的 `WHERE` 或紧跟 `WHERE` 的首个 `AND`/`OR`，
    /// 让用户无需写 `WHERE 1 = 1` 兜底。
    fn cleanup_where_clauses(&self, sql: &str) -> String {
        let s = self
            .where_leading_and_or_regex
            .replace_all(sql, "WHERE")
            .into_owned();
        let s = self
            .where_empty_before_clause_regex
            .replace_all(&s, "$1")
            .into_owned();
        let s = self
            .where_empty_before_terminator_regex
            .replace_all(&s, "$1")
            .into_owned();
        self.where_empty_at_end_regex
            .replace_all(&s, "")
            .into_owned()
    }

    fn process_optional_blocks(
        &self,
        sql: &str,
        parameters: &Option<HashMap<String, String>>,
    ) -> AppResult<String> {
        let result = self.optional_regex.replace_all(sql, |caps: &Captures| {
            let optional_content = &caps[1];

            // 检查可选内容中是否包含参数（缺失 = 未传或值为空字符串）
            let has_missing_params = if let Some(params) = parameters {
                self.parameter_regex.find_iter(optional_content).any(|mat| {
                    let param_name = mat
                        .as_str()
                        .trim_start_matches("{{")
                        .trim_end_matches("}}");
                    params.get(param_name).is_none_or(|v| v.is_empty())
                })
            } else {
                // 如果没有提供参数，检查是否包含任何参数引用
                self.parameter_regex.find_iter(optional_content).count() > 0
            };

            if has_missing_params {
                // 如果有缺失的参数，移除整个可选块
                String::new()
            } else {
                // 如果所有参数都存在，保留内容（但还需要处理参数替换）
                optional_content.to_string()
            }
        });

        Ok(result.to_string())
    }

    fn process_snippets(
        &self,
        sql: &str,
        snippets: &Option<HashMap<String, String>>,
    ) -> AppResult<String> {
        // 引用了不存在的片段名时直接报错。静默移除会产出
        // `FROM ( ) t` 这类坏 SQL，把 typo 暴露在保存/执行期更安全。
        let empty = HashMap::new();
        let snippets_map = snippets.as_ref().unwrap_or(&empty);
        for caps in self.snippet_regex.captures_iter(sql) {
            let name = caps.get(1).map(|c| c.as_str()).unwrap_or_default();
            if !snippets_map.contains_key(name) {
                return Err(AppError::BadRequest(format!(
                    "代码片段不存在: {name}"
                )));
            }
        }

        let result = self.snippet_regex.replace_all(sql, |caps: &Captures| {
            let snippet_name = &caps[1];
            snippets_map.get(snippet_name).cloned().unwrap_or_default()
        });

        Ok(result.to_string())
    }

    fn process_parameters(
        &self,
        sql: &str,
        parameters: &Option<HashMap<String, String>>,
    ) -> AppResult<String> {
        let result =
            self.parameter_regex.replace_all(sql, |caps: &Captures| {
                let param_name = &caps[1];
                match parameters {
                    Some(params) => match params.get(param_name) {
                        Some(value) => Self::quote_param_value(value),
                        None => String::new(),
                    },
                    None => String::new(),
                }
            });

        Ok(result.to_string())
    }

    fn quote_param_value(value: &str) -> String {
        if value.is_empty() {
            return String::new();
        }
        // 已经手动加了引号的，原样返回
        if (value.starts_with('\'') && value.ends_with('\''))
            || (value.starts_with('"') && value.ends_with('"'))
        {
            return value.to_string();
        }
        // 仅当文本本身就是该数字的规范写法时才不加引号。
        // 这样带前导零的 ID(如 "0000003968")会被当作字符串加引号，
        // 不会被 DuckDB 误解析为整数字面量而触发列类型转换错误。
        let is_canonical_number = value
            .parse::<i64>()
            .map(|n| n.to_string() == value)
            .unwrap_or(false)
            || value
                .parse::<f64>()
                .map(|n| n.to_string() == value)
                .unwrap_or(false);
        if is_canonical_number {
            return value.to_string();
        }
        // 布尔值不加引号
        if value.eq_ignore_ascii_case("true")
            || value.eq_ignore_ascii_case("false")
            || value.eq_ignore_ascii_case("null")
        {
            return value.to_string();
        }
        // 其余值自动加单引号，转义内部单引号防止注入
        format!("'{}'", value.replace('\'', "''"))
    }

    /// 提取 `{{param}}` 参数名（去重，保持首次出现顺序；不含 snippet 占位）
    pub fn extract_parameter_names(&self, sql_template: &str) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for caps in self.parameter_regex.captures_iter(sql_template) {
            if let Some(name) = caps.get(1) {
                let n = name.as_str().to_string();
                if seen.insert(n.clone()) {
                    out.push(n);
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser() -> SqlParser {
        SqlParser::new().unwrap()
    }

    fn params(pairs: &[(&str, &str)]) -> Option<HashMap<String, String>> {
        Some(
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        )
    }

    const WHERE_TEMPLATE: &str = "SELECT * FROM t \
        WHERE [[t.a = {{a}}]] \
        [[AND t.b = {{b}}]] \
        [[AND t.c = {{c}}]] \
        GROUP BY t.d";

    #[test]
    fn cleanup_strips_where_when_all_optional_missing() {
        let result =
            parser().parse(WHERE_TEMPLATE, &params(&[]), &None).unwrap();
        assert!(
            !result.to_uppercase().contains("WHERE"),
            "expected WHERE removed, got: {result}"
        );
        assert!(result.contains("GROUP BY t.d"));
    }

    #[test]
    fn cleanup_strips_leading_and_when_first_optional_missing() {
        let result = parser()
            .parse(WHERE_TEMPLATE, &params(&[("b", "x")]), &None)
            .unwrap();
        assert!(result.contains("WHERE t.b = 'x'"), "got: {result}");
        assert!(
            !result.to_uppercase().contains("WHERE  AND"),
            "got: {result}"
        );
    }

    #[test]
    fn cleanup_noop_when_first_optional_present() {
        let result = parser()
            .parse(WHERE_TEMPLATE, &params(&[("a", "x"), ("b", "y")]), &None)
            .unwrap();
        assert!(result.contains("WHERE t.a = 'x'"), "got: {result}");
        assert!(result.contains("AND t.b = 'y'"), "got: {result}");
    }

    #[test]
    fn cleanup_handles_where_at_end_of_sql() {
        let tpl = "DELETE FROM t WHERE [[id = {{id}}]]";
        let result = parser().parse(tpl, &params(&[]), &None).unwrap();
        assert!(!result.to_uppercase().contains("WHERE"), "got: {result}");
    }

    #[test]
    fn cleanup_handles_where_before_close_paren() {
        let tpl = "SELECT * FROM (SELECT * FROM t WHERE [[id = {{id}}]]) sub";
        let result = parser().parse(tpl, &params(&[]), &None).unwrap();
        assert!(!result.to_uppercase().contains("WHERE"), "got: {result}");
        assert!(result.contains(") sub"));
    }

    #[test]
    fn cleanup_handles_where_in_subquery() {
        let tpl = "SELECT * FROM (SELECT x FROM t WHERE [[t.a = {{a}}]] [[AND t.b = {{b}}]] GROUP BY x) sub";
        let result =
            parser().parse(tpl, &params(&[("b", "x")]), &None).unwrap();
        assert!(result.contains("WHERE t.b = 'x'"), "got: {result}");
    }

    #[test]
    fn cleanup_or_chain_also_stripped() {
        let tpl = "SELECT * FROM t WHERE [[t.a = {{a}}]] [[OR t.b = {{b}}]]";
        let result =
            parser().parse(tpl, &params(&[("b", "x")]), &None).unwrap();
        assert!(result.contains("WHERE t.b = 'x'"), "got: {result}");
    }

    #[test]
    fn cleanup_preserves_existing_where_1_eq_1_pattern() {
        let tpl = "SELECT * FROM t WHERE 1 = 1 [[AND t.a = {{a}}]] GROUP BY x";
        let result = parser().parse(tpl, &params(&[]), &None).unwrap();
        assert!(result.contains("WHERE 1 = 1"), "got: {result}");
    }

    /// 前端 formatSql 对含子查询的可选块的输出形态:`[[` 与 `]]` 之间跨多行
    const MULTILINE_TEMPLATE: &str = "SELECT * FROM t3 [[AND t3.in_dept_code IN (\n  SELECT\n    org_code\n  FROM\n    sys_depart\n  WHERE\n    hospital_zone = {{hospitalZone}}\n)]]";

    #[test]
    fn multiline_optional_block_kept_when_params_present() {
        let result = parser()
            .parse(MULTILINE_TEMPLATE, &params(&[("hospitalZone", "A")]), &None)
            .unwrap();
        assert!(result.contains("hospital_zone = 'A'"), "got: {result}");
        assert!(!result.contains("[["), "got: {result}");
        assert!(!result.contains("]]"), "got: {result}");
    }

    #[test]
    fn multiline_optional_block_removed_when_params_missing() {
        let result = parser()
            .parse(MULTILINE_TEMPLATE, &params(&[]), &None)
            .unwrap();
        assert!(!result.contains("[["), "got: {result}");
        assert!(!result.contains("]]"), "got: {result}");
        assert!(!result.to_uppercase().contains("AND"), "got: {result}");
    }

    #[test]
    fn leading_zero_id_is_quoted_as_string() {
        let tpl = "SELECT * FROM t WHERE id = {{id}}";
        let result = parser()
            .parse(tpl, &params(&[("id", "0000003968")]), &None)
            .unwrap();
        assert!(
            result.contains("id = '0000003968'"),
            "leading-zero id must stay a quoted string, got: {result}"
        );
    }

    #[test]
    fn canonical_numbers_stay_unquoted() {
        let tpl = "SELECT * FROM t WHERE a = {{a}} AND b = {{b}} \
            AND c = {{c}}";
        let result = parser()
            .parse(
                tpl,
                &params(&[("a", "3968"), ("b", "-5"), ("c", "0")]),
                &None,
            )
            .unwrap();
        assert!(result.contains("a = 3968"), "got: {result}");
        assert!(result.contains("b = -5"), "got: {result}");
        assert!(result.contains("c = 0"), "got: {result}");
    }

    #[test]
    fn non_canonical_numeric_strings_are_quoted() {
        let tpl = "SELECT * FROM t WHERE a = {{a}} AND b = {{b}}";
        let result = parser()
            .parse(tpl, &params(&[("a", "nan"), ("b", "1e5")]), &None)
            .unwrap();
        assert!(result.contains("a = 'nan'"), "got: {result}");
        assert!(result.contains("b = '1e5'"), "got: {result}");
    }

    fn snippets(pairs: &[(&str, &str)]) -> Option<HashMap<String, String>> {
        Some(
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        )
    }

    #[test]
    fn snippet_optional_blocks_evaluated_when_param_present() {
        // 片段内含 [[...]] 可选块：参数齐全时保留并完成参数替换
        let snips = snippets(&[(
            "main",
            "FROM (SELECT * FROM t WHERE 1 = 1 [[AND t.a = {{a}}]]) sub",
        )]);
        let result = parser()
            .parse(
                "SELECT count(1) {{snippet:main}}",
                &params(&[("a", "x")]),
                &snips,
            )
            .unwrap();
        assert!(result.contains("AND t.a = 'x'"), "got: {result}");
        assert!(!result.contains("[["), "got: {result}");
        assert!(!result.contains("{{"), "got: {result}");
    }

    #[test]
    fn snippet_optional_blocks_removed_when_param_missing() {
        let snips = snippets(&[(
            "main",
            "FROM (SELECT * FROM t WHERE 1 = 1 [[AND t.a = {{a}}]]) sub",
        )]);
        let result = parser()
            .parse("SELECT count(1) {{snippet:main}}", &params(&[]), &snips)
            .unwrap();
        assert!(!result.contains("[["), "got: {result}");
        assert!(!result.contains("AND t.a"), "got: {result}");
    }

    #[test]
    fn snippet_where_residue_cleaned_when_all_optional_missing() {
        // 片段子查询的 WHERE 后只跟可选块，全部缺失时应清理残留 WHERE
        let snips = snippets(&[(
            "main",
            "FROM (SELECT * FROM t WHERE [[t.a = {{a}}]]) sub",
        )]);
        let result = parser()
            .parse("SELECT count(1) {{snippet:main}}", &params(&[]), &snips)
            .unwrap();
        assert!(!result.to_uppercase().contains("WHERE"), "got: {result}");
        assert!(result.contains(") sub"), "got: {result}");
    }

    #[test]
    fn missing_snippet_is_an_error() {
        let snips = snippets(&[]);
        let err = parser()
            .parse("SELECT 1 {{snippet:nope}}", &params(&[]), &snips)
            .unwrap_err();
        assert!(err.to_string().contains("nope"), "got: {err}");
    }

    #[test]
    fn snippet_ref_with_no_snippet_map_is_an_error() {
        let err = parser()
            .parse("SELECT 1 {{snippet:foo}}", &params(&[]), &None)
            .unwrap_err();
        assert!(err.to_string().contains("foo"), "got: {err}");
    }

    #[test]
    fn nested_snippets_expand() {
        let snips = snippets(&[
            ("inner", "t.a = {{a}}"),
            ("outer", "FROM t WHERE {{snippet:inner}}"),
        ]);
        let result = parser()
            .parse("SELECT 1 {{snippet:outer}}", &params(&[("a", "x")]), &snips)
            .unwrap();
        assert!(result.contains("t.a = 'x'"), "got: {result}");
        assert!(!result.contains("snippet"), "got: {result}");
    }

    #[test]
    fn cyclic_snippet_is_an_error() {
        let snips = snippets(&[("a_snip", "SELECT {{snippet:a_snip}}")]);
        let err = parser()
            .parse("{{snippet:a_snip}}", &params(&[]), &snips)
            .unwrap_err();
        assert!(
            err.to_string().contains("循环引用")
                || err.to_string().contains("嵌套"),
            "got: {err}"
        );
    }

    #[test]
    fn optional_block_containing_snippet_ref() {
        // 可选块内含片段引用：参数缺失时整块（含片段）剔除，
        // 参数齐全时片段展开并替换参数
        let snips = snippets(&[("cond", "t.a = {{a}}")]);
        let kept = parser()
            .parse(
                "SELECT * FROM t WHERE 1 = 1 [[AND {{snippet:cond}}]]",
                &params(&[("a", "x")]),
                &snips,
            )
            .unwrap();
        assert!(kept.contains("AND t.a = 'x'"), "got: {kept}");

        let removed = parser()
            .parse(
                "SELECT * FROM t WHERE 1 = 1 [[AND {{snippet:cond}}]]",
                &params(&[]),
                &snips,
            )
            .unwrap();
        assert!(!removed.contains("t.a"), "got: {removed}");
        assert!(!removed.contains("[["), "got: {removed}");
    }
}
