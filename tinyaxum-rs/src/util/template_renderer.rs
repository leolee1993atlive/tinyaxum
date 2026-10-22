use std::collections::HashMap;

use handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext,
    RenderErrorReason,
};
use serde_json::{Map, Value};

use crate::common::vo::sql_run::SqlRunResultVo;

#[derive(Clone)]
pub struct TemplateRenderer {
    engine: Handlebars<'static>,
}

impl TemplateRenderer {
    pub fn new() -> Self {
        let mut engine = Handlebars::new();
        engine.set_strict_mode(true);
        engine.register_helper(
            "tojson",
            Box::new(
                |h: &Helper,
                 _: &Handlebars,
                 _: &Context,
                 _: &mut RenderContext,
                 out: &mut dyn Output|
                 -> HelperResult {
                    let param = h.param(0).ok_or_else(|| {
                        RenderErrorReason::ParamNotFoundForIndex("tojson", 0)
                    })?;
                    let json = serde_json::to_string(param.value())
                        .unwrap_or_default();
                    out.write(&json)?;
                    Ok(())
                },
            ),
        );
        // 工具函数:把 serde_json::Value 解释为 f64,失败返回 None
        fn as_f64(v: &Value) -> Option<f64> {
            match v {
                Value::Number(n) => n.as_f64(),
                Value::String(s) => s.parse().ok(),
                _ => None,
            }
        }

        // 工具函数:把 f64 写成 JSON 数字字面量(整数省略小数位)
        fn write_num(out: &mut dyn Output, n: f64) -> HelperResult {
            if n.is_finite() && n.fract() == 0.0 && n.abs() < 1e16 {
                out.write(&format!("{}", n as i64))?;
            } else {
                out.write(&format!("{}", n))?;
            }
            Ok(())
        }

        engine.register_helper(
            "changeRate",
            Box::new(
                |h: &Helper,
                 _: &Handlebars,
                 _: &Context,
                 _: &mut RenderContext,
                 out: &mut dyn Output|
                 -> HelperResult {
                    let cur = h.param(0).and_then(|p| as_f64(p.value()));
                    let base = h.param(1).and_then(|p| as_f64(p.value()));
                    match (cur, base) {
                        (Some(c), Some(b)) if b != 0.0 => {
                            write_num(out, (c - b) / b)
                        }
                        _ => {
                            out.write("null")?;
                            Ok(())
                        }
                    }
                },
            ),
        );

        engine.register_helper(
            "changeRatePct",
            Box::new(
                |h: &Helper,
                 _: &Handlebars,
                 _: &Context,
                 _: &mut RenderContext,
                 out: &mut dyn Output|
                 -> HelperResult {
                    let cur = h.param(0).and_then(|p| as_f64(p.value()));
                    let base = h.param(1).and_then(|p| as_f64(p.value()));
                    match (cur, base) {
                        (Some(c), Some(b)) if b != 0.0 => {
                            write_num(out, (c - b) / b * 100.0)
                        }
                        _ => {
                            out.write("null")?;
                            Ok(())
                        }
                    }
                },
            ),
        );

        engine.register_helper(
            "delta",
            Box::new(
                |h: &Helper,
                 _: &Handlebars,
                 _: &Context,
                 _: &mut RenderContext,
                 out: &mut dyn Output|
                 -> HelperResult {
                    let cur = h.param(0).and_then(|p| as_f64(p.value()));
                    let base = h.param(1).and_then(|p| as_f64(p.value()));
                    match (cur, base) {
                        (Some(c), Some(b)) => write_num(out, c - b),
                        _ => {
                            out.write("null")?;
                            Ok(())
                        }
                    }
                },
            ),
        );

        engine.register_helper(
            "fmtPct",
            Box::new(
                |h: &Helper,
                 _: &Handlebars,
                 _: &Context,
                 _: &mut RenderContext,
                 out: &mut dyn Output|
                 -> HelperResult {
                    let ratio = h.param(0).and_then(|p| as_f64(p.value()));
                    let decimals = h
                        .param(1)
                        .and_then(|p| p.value().as_u64())
                        .unwrap_or(1)
                        as usize;
                    match ratio {
                        Some(r) if r.is_finite() => {
                            let s = format!("\"{:.*}%\"", decimals, r * 100.0);
                            out.write(&s)?;
                        }
                        _ => {
                            out.write("null")?;
                        }
                    }
                    Ok(())
                },
            ),
        );

        engine.register_helper(
            "default",
            Box::new(
                |h: &Helper,
                 _: &Handlebars,
                 _: &Context,
                 _: &mut RenderContext,
                 out: &mut dyn Output|
                 -> HelperResult {
                    let v =
                        h.param(0).map(|p| p.value()).unwrap_or(&Value::Null);
                    let fallback =
                        h.param(1).map(|p| p.value()).unwrap_or(&Value::Null);
                    let pick = if v.is_null() { fallback } else { v };
                    let json = serde_json::to_string(pick).unwrap_or_default();
                    out.write(&json)?;
                    Ok(())
                },
            ),
        );

        Self { engine }
    }

    /// Render a template. Double-brace expressions (`{{expr}}`) are
    /// automatically rewritten to `{{{tojson expr}}}` before rendering, so
    /// objects and arrays serialize correctly without any extra syntax.
    /// Triple-brace expressions and Handlebars block tags are left untouched.
    pub fn render_template(
        &self,
        template: &str,
        data: &Value,
    ) -> anyhow::Result<String> {
        let preprocessed = Self::preprocess(template);
        self.engine
            .render_template(&preprocessed, data)
            .map_err(|e| anyhow::anyhow!("Template render error: {}", e))
    }

    /// Replace `{{expr}}` → `{{{tojson expr}}}`.
    /// Leaves `{{{…}}}` and block tags (`#`, `/`, `!`, `>`, `^`) intact.
    fn preprocess(template: &str) -> String {
        let mut out =
            String::with_capacity(template.len() + template.len() / 4);
        let mut rest = template;
        while let Some(open) = rest.find("{{") {
            out.push_str(&rest[..open]);
            rest = &rest[open + 2..];
            // Triple brace or block tag — emit '{{' and keep scanning.
            if let Some('{' | '#' | '/' | '!' | '>' | '^') = rest.chars().next()
            {
                out.push_str("{{");
                continue;
            }
            // Special block keywords: else, else if — keep as-is.
            if rest.starts_with("else") {
                out.push_str("{{");
                continue;
            }
            // Find the matching '}}' and wrap in tojson.
            if let Some(close) = rest.find("}}") {
                let expr = rest[..close].trim();
                // Helper call: contains a space or subexpression — wrap in triple-brace
                // to bypass Handlebars HTML escaping (helper writes raw JSON literals).
                if expr.contains(' ') || expr.contains('(') {
                    out.push_str("{{{");
                    out.push_str(&rest[..close]);
                    out.push_str("}}}");
                } else {
                    // Plain data access: wrap in tojson so output is a JSON literal.
                    out.push_str("{{{tojson ");
                    out.push_str(expr);
                    out.push_str("}}}");
                }
                rest = &rest[close + 2..];
            } else {
                out.push_str("{{");
            }
        }
        out.push_str(rest);
        out
    }

    pub fn build_template_data(
        &self,
        query_results: HashMap<
            &str,
            crate::common::vo::sql_run::QueryResultBundle,
        >,
    ) -> Value {
        // 取一次 result 的"单行/数组"投影
        fn project_rows(result: &SqlRunResultVo) -> Value {
            let rows: Vec<Map<String, Value>> = result.rows.clone();
            if result.is_collection == 0 && rows.len() == 1 {
                match rows.into_iter().next() {
                    Some(row) => Value::Object(row),
                    None => Value::Null,
                }
            } else {
                Value::Array(rows.into_iter().map(Value::Object).collect())
            }
        }

        // 构造一个与 current shape 相同、字段值全 null 的占位对象
        fn null_shadow(current: &Value) -> Value {
            match current {
                Value::Object(m) => {
                    let mut out = Map::new();
                    for k in m.keys() {
                        out.insert(k.clone(), Value::Null);
                    }
                    Value::Object(out)
                }
                Value::Array(_) => Value::Array(Vec::new()),
                _ => Value::Null,
            }
        }

        let mut root = Map::new();
        for (alias, bundle) in query_results {
            let current_value = project_rows(&bundle.current);

            // yoy / mom:成功且非空 → 投影;否则使用 null_shadow(current_value) 兜底
            let yoy_value = match &bundle.yoy {
                Some(r) if !r.rows.is_empty() => project_rows(r),
                _ => null_shadow(&current_value),
            };
            let mom_value = match &bundle.mom {
                Some(r) if !r.rows.is_empty() => project_rows(r),
                _ => null_shadow(&current_value),
            };

            root.insert(alias.to_string(), current_value);
            root.insert(format!("{alias}_yoy"), yoy_value);
            root.insert(format!("{alias}_mom"), mom_value);

            // 分页元数据仅基于 current
            root.insert(
                format!("{alias}_total"),
                Value::Number(bundle.current.total.into()),
            );
            root.insert(
                format!("{alias}_current"),
                Value::Number(bundle.current.current.into()),
            );
            root.insert(
                format!("{alias}_size"),
                Value::Number(bundle.current.size.into()),
            );
            root.insert(
                format!("{alias}_pages"),
                Value::Number(bundle.current.pages.into()),
            );
        }
        Value::Object(root)
    }
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn render(template: &str, data: Value) -> String {
        TemplateRenderer::new()
            .render_template(template, &data)
            .unwrap()
    }

    // ---- preprocess ----

    #[test]
    fn preprocess_wraps_plain_data_access_in_tojson() {
        let out = TemplateRenderer::preprocess("{{a.b}}");
        assert_eq!(out, "{{{tojson a.b}}}");
    }

    #[test]
    fn preprocess_wraps_helper_call_in_triple_brace() {
        let out = TemplateRenderer::preprocess("{{helper x y}}");
        assert_eq!(out, "{{{helper x y}}}");
    }

    #[test]
    fn preprocess_wraps_subexpression_in_triple_brace() {
        let out = TemplateRenderer::preprocess("{{outer (inner a) b}}");
        assert_eq!(out, "{{{outer (inner a) b}}}");
    }

    #[test]
    fn preprocess_keeps_triple_brace_as_is() {
        let out = TemplateRenderer::preprocess("{{{rawValue}}}");
        assert_eq!(out, "{{{rawValue}}}");
    }

    #[test]
    fn preprocess_keeps_block_tags_as_is() {
        let out = TemplateRenderer::preprocess("{{#if x}}A{{else}}B{{/if}}");
        assert_eq!(out, "{{#if x}}A{{else}}B{{/if}}");
    }

    // ---- changeRate ----

    #[test]
    fn change_rate_normal() {
        let out = render(r#"{{changeRate a b}}"#, json!({ "a": 100, "b": 80 }));
        assert_eq!(out, "0.25");
    }

    #[test]
    fn change_rate_zero_base_is_null() {
        let out = render(r#"{{changeRate a b}}"#, json!({ "a": 100, "b": 0 }));
        assert_eq!(out, "null");
    }

    #[test]
    fn change_rate_null_arg_is_null() {
        let out =
            render(r#"{{changeRate a b}}"#, json!({ "a": 100, "b": null }));
        assert_eq!(out, "null");
    }

    // ---- changeRatePct ----

    #[test]
    fn change_rate_pct_normal_integer() {
        let out =
            render(r#"{{changeRatePct a b}}"#, json!({ "a": 100, "b": 80 }));
        assert_eq!(out, "25");
    }

    #[test]
    fn change_rate_pct_null_safe() {
        let out =
            render(r#"{{changeRatePct a b}}"#, json!({ "a": null, "b": 80 }));
        assert_eq!(out, "null");
    }

    // ---- delta ----

    #[test]
    fn delta_normal() {
        let out = render(r#"{{delta a b}}"#, json!({ "a": 100, "b": 80 }));
        assert_eq!(out, "20");
    }

    #[test]
    fn delta_null_safe() {
        let out = render(r#"{{delta a b}}"#, json!({ "a": 100, "b": null }));
        assert_eq!(out, "null");
    }

    // ---- fmtPct ----

    #[test]
    fn fmt_pct_default_one_decimal() {
        let out = render(r#"{{fmtPct r}}"#, json!({ "r": 0.1 }));
        assert_eq!(out, "\"10.0%\"");
    }

    #[test]
    fn fmt_pct_zero_decimal() {
        // 选 0.1 这种 r * 100 = 10.0 整数刚好的值,避开 0.105 这种
        // 浮点舍入(0.105 在 f64 里实际是 0.1049999...,{:.0} 会变 "10" 不是 "11")。
        let out = render(r#"{{fmtPct r 0}}"#, json!({ "r": 0.1 }));
        assert_eq!(out, "\"10%\"");
    }

    #[test]
    fn fmt_pct_two_decimal() {
        let out = render(r#"{{fmtPct r 2}}"#, json!({ "r": 0.1234 }));
        assert_eq!(out, "\"12.34%\"");
    }

    #[test]
    fn fmt_pct_null_safe() {
        let out = render(r#"{{fmtPct r}}"#, json!({ "r": null }));
        assert_eq!(out, "null");
    }

    // ---- default ----

    #[test]
    fn default_returns_value_when_present() {
        let out = render(r#"{{default v fb}}"#, json!({ "v": 5, "fb": 0 }));
        assert_eq!(out, "5");
    }

    #[test]
    fn default_returns_fallback_when_null() {
        let out = render(r#"{{default v fb}}"#, json!({ "v": null, "fb": 0 }));
        assert_eq!(out, "0");
    }

    // ---- 子表达式探针(决定 spec 方案 A vs 方案 B)----

    #[test]
    fn subexpression_probe_change_rate_then_fmt_pct() {
        // 如果本测试 panic,意味着 handlebars-rust 在 strict_mode 下
        // 不支持子表达式语法,需要回退到 spec 方案 A 并补 changeRateText helper。
        // 如果通过,spec 升级允许方案 B。
        let result = TemplateRenderer::new().render_template(
            r#"{{fmtPct (changeRate a b) 1}}"#,
            &json!({ "a": 100, "b": 80 }),
        );
        assert!(result.is_ok(), "subexpression failed: {:?}", result.err());
        assert_eq!(result.unwrap(), "\"25.0%\"");
    }

    #[test]
    fn build_template_data_single_row_with_yoy_mom() {
        use crate::common::vo::sql_run::{
            QueryResultBundle, SqlResultColumnVo, SqlRunResultVo,
        };
        fn row(amt: i64) -> Map<String, Value> {
            let mut m = Map::new();
            m.insert("amt".into(), json!(amt));
            m
        }
        fn vo(amt: i64) -> SqlRunResultVo {
            SqlRunResultVo {
                columns: vec![SqlResultColumnVo {
                    name: "amt".into(),
                    physical_type: "BIGINT".into(),
                }],
                rows: vec![row(amt)],
                total: 1,
                current: 1,
                size: 10,
                pages: 1,
                is_collection: 0,
            }
        }
        let renderer = TemplateRenderer::new();
        let mut bundles = HashMap::new();
        bundles.insert(
            "revenue",
            QueryResultBundle {
                current: vo(100),
                yoy: Some(vo(80)),
                mom: Some(vo(90)),
            },
        );
        let data = renderer.build_template_data(bundles);
        assert_eq!(data["revenue"]["amt"], json!(100));
        assert_eq!(data["revenue_yoy"]["amt"], json!(80));
        assert_eq!(data["revenue_mom"]["amt"], json!(90));
        assert_eq!(data["revenue_total"], json!(1));
    }

    #[test]
    fn build_template_data_yoy_failure_uses_null_shadow() {
        use crate::common::vo::sql_run::{
            QueryResultBundle, SqlResultColumnVo, SqlRunResultVo,
        };
        let mut row = Map::new();
        row.insert("amt".into(), json!(100));
        row.insert("count".into(), json!(5));
        let current = SqlRunResultVo {
            columns: vec![
                SqlResultColumnVo {
                    name: "amt".into(),
                    physical_type: "BIGINT".into(),
                },
                SqlResultColumnVo {
                    name: "count".into(),
                    physical_type: "BIGINT".into(),
                },
            ],
            rows: vec![row],
            total: 1,
            current: 1,
            size: 10,
            pages: 1,
            is_collection: 0,
        };
        let mut bundles = HashMap::new();
        bundles.insert(
            "revenue",
            QueryResultBundle {
                current,
                yoy: None,
                mom: None,
            },
        );
        let data = TemplateRenderer::new().build_template_data(bundles);
        assert_eq!(data["revenue_yoy"]["amt"], Value::Null);
        assert_eq!(data["revenue_yoy"]["count"], Value::Null);
        assert_eq!(data["revenue_mom"]["amt"], Value::Null);
    }
}
