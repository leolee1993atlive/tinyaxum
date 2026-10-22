use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use validator::Validate;

use crate::{
    biz::sqlquery::model::{
        sql_query_param_field::SqlQueryParamField,
        sql_query_result_field::SqlQueryResultField,
    },
    common::{
        model::entity::{BaseEntity, BaseVo},
        vo::page::{PageQuery, deserialize_opt_i64_from_str},
    },
    error::error::{AppError, AppResult},
    middleware::extractors::CurrentUser,
};

fn default_is_collection() -> i8 {
    1
}

/// MySQL `JSON` / PG `JSONB` 等驱动常反序列化为对象；SQLite `TEXT` 常为 JSON 字符串。
fn deserialize_viz_config_column_opt<'de, D>(
    deserializer: D,
) -> Result<Option<Value>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
        Str(String),
        Json(Value),
    }
    match Option::<Raw>::deserialize(deserializer)? {
        None => Ok(None),
        Some(Raw::Str(s)) => {
            let t = s.trim();
            if t.is_empty() {
                Ok(None)
            } else {
                serde_json::from_str(t)
                    .map(Some)
                    .map_err(serde::de::Error::custom)
            }
        }
        Some(Raw::Json(v)) => {
            if v.is_null() {
                Ok(None)
            } else {
                Ok(Some(v))
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SqlQuery {
    pub sql_query_id: Option<i64>,
    pub sql_query_name: String,
    pub sql_query_desc: Option<String>,
    pub sql_query_content: String,
    pub data_source_id: Option<i64>,
    pub result_meta_updated_at: Option<DateTime<Utc>>,
    /// 是否按多条（集合）结果设计；各库统一存整数 0/1
    pub is_collection: i8,
    #[serde(default, deserialize_with = "deserialize_viz_config_column_opt")]
    pub viz_config: Option<Value>,
    pub module: Option<String>,
    pub category: Option<String>,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}

impl SqlQuery {
    pub fn from_create_bo(bo: SqlQueryCreateBo, user: &CurrentUser) -> Self {
        SqlQuery {
            sql_query_id: None,
            sql_query_name: bo.sql_query_name,
            sql_query_desc: bo.sql_query_desc,
            sql_query_content: bo.sql_query_content,
            data_source_id: bo.data_source_id,
            result_meta_updated_at: None,
            is_collection: bo.is_collection,
            viz_config: bo.viz_config.clone(),
            module: bo.module,
            category: bo.category,
            base_entity: BaseEntity::new(user.get_user_id()),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_update_bo(&mut self, bo: SqlQueryUpdateBo, user: &CurrentUser) {
        self.sql_query_id = bo.sql_query_id;
        self.sql_query_name = bo.sql_query_name;
        self.sql_query_desc = bo.sql_query_desc;
        self.sql_query_content = bo.sql_query_content;
        self.data_source_id = bo.data_source_id;
        self.is_collection = bo.is_collection;
        self.viz_config = bo.viz_config.clone();
        self.module = bo.module;
        self.category = bo.category;
        self.base_entity.update(user.get_user_id());
    }

    pub fn to_detail_vo(&self) -> SqlQueryDetailVo {
        let viz_config =
            self.viz_config.as_ref().filter(|v| !v.is_null()).cloned();
        SqlQueryDetailVo {
            sql_query_id: self.sql_query_id.unwrap_or_default(),
            sql_query_name: self.sql_query_name.clone(),
            sql_query_desc: self.sql_query_desc.clone(),
            sql_query_content: self.sql_query_content.clone(),
            data_source_id: self.data_source_id.unwrap_or_default(),
            result_meta_updated_at: self.result_meta_updated_at,
            is_collection: self.is_collection,
            viz_config,
            module: self.module.clone(),
            category: self.category.clone(),
            result_fields: Vec::new(),
            param_fields: Vec::new(),
            base_entity: BaseVo::new(self.base_entity.clone()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SqlQueryPageQuery {
    #[serde(flatten)]
    #[validate(nested)]
    pub page: PageQuery,
    pub keyword: Option<String>,
    pub sql_query_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_opt_i64_from_str")]
    pub data_source_id: Option<i64>,
    pub module: Option<String>,
    pub category: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SqlQueryCreateBo {
    #[validate(length(min = 1, message = "sql_query_name cannot be empty"))]
    pub sql_query_name: String,
    pub sql_query_desc: Option<String>,
    #[validate(length(
        min = 1,
        message = "sql_query_content cannot be empty"
    ))]
    pub sql_query_content: String,
    #[validate(required)]
    pub data_source_id: Option<i64>,
    /// 0 单行/标量，1 多条（集合）；缺省为 1
    #[serde(default = "default_is_collection")]
    #[validate(range(min = 0, max = 1))]
    pub is_collection: i8,
    #[serde(default)]
    pub viz_config: Option<Value>,
    pub module: Option<String>,
    pub category: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SqlQueryUpdateBo {
    #[validate(required)]
    pub sql_query_id: Option<i64>,
    #[validate(length(min = 1, message = "sql_query_name cannot be empty"))]
    pub sql_query_name: String,
    pub sql_query_desc: Option<String>,
    #[validate(length(
        min = 1,
        message = "sql_query_content cannot be empty"
    ))]
    pub sql_query_content: String,
    #[validate(required)]
    pub data_source_id: Option<i64>,
    #[validate(range(min = 0, max = 1))]
    pub is_collection: i8,
    #[serde(default)]
    pub viz_config: Option<Value>,
    pub module: Option<String>,
    pub category: Option<String>,
}

/// 列表/分页查询映射行（不含大字段 `sql_query_content`、`viz_config` 等;
/// 但保留从 `viz_config` 解析出的 `viz_kind` 摘要,用于前端筛选候选)。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SqlQueryListRow {
    pub sql_query_id: Option<i64>,
    pub sql_query_name: String,
    pub sql_query_desc: Option<String>,
    pub data_source_id: Option<i64>,
    pub is_collection: i8,
    /// 'scalar' / 'table' / 'chart' / None;非集合一律 'scalar';
    /// 集合且 viz_config.kind 缺失视为 'table'(向后兼容)
    pub viz_kind: Option<String>,
    pub module: Option<String>,
    pub category: Option<String>,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}

impl SqlQueryListRow {
    pub fn to_list_vo(&self) -> SqlQueryListVo {
        SqlQueryListVo {
            sql_query_id: self.sql_query_id.unwrap_or_default(),
            sql_query_name: self.sql_query_name.clone(),
            sql_query_desc: self.sql_query_desc.clone(),
            data_source_id: self.data_source_id.unwrap_or_default(),
            is_collection: self.is_collection,
            viz_kind: self.viz_kind.clone(),
            module: self.module.clone(),
            category: self.category.clone(),
            base_entity: BaseVo::new(self.base_entity.clone()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SqlQueryListVo {
    pub sql_query_id: i64,
    pub sql_query_name: String,
    pub sql_query_desc: Option<String>,
    pub data_source_id: i64,
    pub is_collection: i8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viz_kind: Option<String>,
    pub module: Option<String>,
    pub category: Option<String>,
    #[serde(flatten)]
    pub base_entity: BaseVo,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SqlQueryDetailVo {
    pub sql_query_id: i64,
    pub sql_query_name: String,
    pub sql_query_desc: Option<String>,
    pub sql_query_content: String,
    pub data_source_id: i64,
    pub result_meta_updated_at: Option<DateTime<Utc>>,
    pub is_collection: i8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viz_config: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default)]
    pub result_fields: Vec<SqlQueryResultFieldVo>,
    #[serde(default)]
    pub param_fields: Vec<SqlQueryParamFieldVo>,
    #[serde(flatten)]
    pub base_entity: BaseVo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlQueryResultFieldVo {
    pub result_field_id: i64,
    pub field_name: String,
    pub comment_label: Option<String>,
    pub field_type: String,
    pub format_pattern: Option<String>,
    pub field_desc: Option<String>,
    #[serde(flatten)]
    pub base: BaseVo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlQueryParamFieldVo {
    pub param_field_id: i64,
    pub field_name: String,
    pub comment_label: Option<String>,
    pub field_type: String,
    pub format_pattern: Option<String>,
    pub field_desc: Option<String>,
    pub required: i32,
    #[serde(flatten)]
    pub base: BaseVo,
}

/// 仅更新 `viz_config`（JSON），不影响 SQL 正文与其它字段。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlQueryVizConfigSaveBo {
    /// 省略或 `null` 表示清空库中配置。
    #[serde(default)]
    pub viz_config: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SqlQueryRunAdhocBo {
    #[validate(length(
        min = 1,
        message = "sql_query_content cannot be empty"
    ))]
    pub sql_query_content: String,
    pub data_source_id: Option<i64>,
    pub params: Option<HashMap<String, String>>,
    pub page_no: Option<i64>,
    pub page_size: Option<i64>,
}

impl SqlQueryResultField {
    pub fn to_vo(&self) -> SqlQueryResultFieldVo {
        SqlQueryResultFieldVo {
            result_field_id: self.result_field_id.unwrap_or_default(),
            field_name: self.field_name.clone(),
            comment_label: self.comment_label.clone(),
            field_type: self.field_type.clone(),
            format_pattern: self.format_pattern.clone(),
            field_desc: self.field_desc.clone(),
            base: BaseVo::new(self.base_entity.clone()),
        }
    }
}

impl SqlQueryParamField {
    pub fn to_vo(&self) -> SqlQueryParamFieldVo {
        SqlQueryParamFieldVo {
            param_field_id: self.param_field_id.unwrap_or_default(),
            field_name: self.field_name.clone(),
            comment_label: self.comment_label.clone(),
            field_type: self.field_type.clone(),
            format_pattern: self.format_pattern.clone(),
            field_desc: self.field_desc.clone(),
            required: self.required_flag,
            base: BaseVo::new(self.base_entity.clone()),
        }
    }
}

/// 校验前端提交的可视化配置 JSON。
pub fn validate_viz_config_json(v: Option<&Value>) -> AppResult<()> {
    let Some(val) = v else {
        return Ok(());
    };
    let Some(obj) = val.as_object() else {
        return Err(AppError::BadRequest(
            "vizConfig 必须为 JSON 对象".to_string(),
        ));
    };
    if let Some(ver) = obj.get("version")
        && !ver.is_null()
        && ver.as_u64() != Some(1)
    {
        return Err(AppError::BadRequest(
            "vizConfig.version 仅支持 1".to_string(),
        ));
    }
    if let Some(sf) = obj.get("scalarField")
        && !sf.is_null()
        && (!sf.is_string() || sf.as_str().map(str::is_empty).unwrap_or(true))
    {
        return Err(AppError::BadRequest(
            "scalarField 须为非空字符串".to_string(),
        ));
    }
    if let Some(k) = obj.get("kind")
        && !k.is_null()
    {
        let Some(ks) = k.as_str() else {
            return Err(AppError::BadRequest("kind 须为字符串".to_string()));
        };
        if !matches!(ks, "table" | "chart") {
            return Err(AppError::BadRequest(
                "kind 仅支持 table 或 chart".to_string(),
            ));
        }
    }
    if let Some(table) = obj.get("table") {
        if table.is_null() {
            return Ok(());
        }
        let Some(t) = table.as_object() else {
            return Err(AppError::BadRequest("table 须为对象".to_string()));
        };
        let arr = match t.get("columns") {
            None => &[],
            Some(Value::Array(a)) => a.as_slice(),
            Some(_) => {
                return Err(AppError::BadRequest(
                    "table.columns 须为数组".to_string(),
                ));
            }
        };
        for (i, c) in arr.iter().enumerate() {
            let Some(co) = c.as_object() else {
                return Err(AppError::BadRequest(format!(
                    "table.columns[{i}] 须为对象"
                )));
            };
            let fn_ok = co
                .get("fieldName")
                .and_then(|x| x.as_str())
                .is_some_and(|s| !s.is_empty());
            if !fn_ok {
                return Err(AppError::BadRequest(format!(
                    "table.columns[{i}].fieldName 无效"
                )));
            }
            if let Some(a) = co.get("align").and_then(|x| x.as_str())
                && !matches!(a, "left" | "center" | "right")
            {
                return Err(AppError::BadRequest(format!(
                    "table.columns[{i}].align 无效"
                )));
            }
        }
    }
    if let Some(chart) = obj.get("chart") {
        if chart.is_null() {
            return Ok(());
        }
        let Some(c) = chart.as_object() else {
            return Err(AppError::BadRequest("chart 须为对象".to_string()));
        };
        let xf_ok = c
            .get("xField")
            .and_then(|x| x.as_str())
            .is_some_and(|s| !s.is_empty());
        if !xf_ok {
            return Err(AppError::BadRequest(
                "chart.xField 须为非空字符串".to_string(),
            ));
        }
        let Some(series_val) = c.get("series") else {
            return Ok(());
        };
        let Some(arr) = series_val.as_array() else {
            return Err(AppError::BadRequest(
                "chart.series 须为数组".to_string(),
            ));
        };
        for (i, s) in arr.iter().enumerate() {
            let Some(so) = s.as_object() else {
                return Err(AppError::BadRequest(format!(
                    "chart.series[{i}] 须为对象"
                )));
            };
            let fn_ok = so
                .get("fieldName")
                .and_then(|x| x.as_str())
                .is_some_and(|s| !s.is_empty());
            if !fn_ok {
                return Err(AppError::BadRequest(format!(
                    "chart.series[{i}].fieldName 无效"
                )));
            }
            if let Some(t) = so.get("type").and_then(|x| x.as_str())
                && !matches!(t, "line" | "area")
            {
                return Err(AppError::BadRequest(format!(
                    "chart.series[{i}].type 仅支持 line 或 area"
                )));
            }
        }
    }
    Ok(())
}
