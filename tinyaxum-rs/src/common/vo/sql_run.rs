use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlRunResultVo {
    pub columns: Vec<SqlResultColumnVo>,
    pub rows: Vec<Map<String, Value>>,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub current: i64,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub pages: i64,
    /// 是否按多条(集合)结果设计;0 单行/标量,1 多条。
    /// 由 service 层从对应 sql_query 配置带回,adhoc 路径默认 1。
    #[serde(default)]
    pub is_collection: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlResultColumnVo {
    pub name: String,
    pub physical_type: String,
}

/// 一次 HTTP API 编排里,某 alias 对应的当前期 / 同比期 / 环比期三组结果。
/// yoy / mom 为 None 表示未触发对比 或 对比期 SQL 执行失败 或 0 行(由 build_template_data 统一兜底)。
#[derive(Debug, Clone)]
pub struct QueryResultBundle {
    pub current: SqlRunResultVo,
    pub yoy: Option<SqlRunResultVo>,
    pub mom: Option<SqlRunResultVo>,
}
