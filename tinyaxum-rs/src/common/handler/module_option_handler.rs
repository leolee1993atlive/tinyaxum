use std::{collections::BTreeMap, sync::Arc};

use axum::extract::State;
use sea_orm::{FromQueryResult, Statement};
use serde::Serialize;

use crate::{app::AppState, common::vo::response::R};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleOptionVo {
    pub name: String,
    pub categories: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleOptionsResponse {
    pub modules: Vec<ModuleOptionVo>,
}

#[derive(FromQueryResult)]
struct Row {
    module: String,
    category: Option<String>,
}

pub async fn get_module_options(
    State(state): State<Arc<AppState>>,
) -> R<ModuleOptionsResponse> {
    let db = &state.infra.db;

    let backend = db.get_database_backend();
    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();

    // 分别查询两张表再在内存中合并，避免 UNION 在 MySQL 下因两表列排序规则
    // (collation) 不一致触发 "Illegal mix of collations" 错误。表名是硬编码
    // 常量，无注入风险。
    for table in ["biz_sql_query", "biz_http_api"] {
        let sql = format!(
            "SELECT DISTINCT module, category FROM {table} \
             WHERE module IS NOT NULL AND deleted_by IS NULL"
        );

        let rows =
            match Row::find_by_statement(Statement::from_string(backend, sql))
                .all(db)
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(error = %e, "查询 module 选项失败");
                    return R::error_with_code_and_message(
                        500,
                        format!("查询 module 选项失败: {}", e),
                    );
                }
            };

        for r in rows {
            let cats = map.entry(r.module).or_default();
            if let Some(cat) = r.category
                && !cat.trim().is_empty()
                && !cats.contains(&cat)
            {
                cats.push(cat);
            }
        }
    }

    let modules = map
        .into_iter()
        .map(|(name, categories)| ModuleOptionVo { name, categories })
        .collect();

    R::ok_with_data(ModuleOptionsResponse { modules })
}
