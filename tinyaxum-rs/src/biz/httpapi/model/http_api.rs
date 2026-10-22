use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;

use super::http_api_query::{HttpApiQueryCreateBo, HttpApiQueryVo};
use crate::common::model::entity::{BaseEntity, BaseVo};

fn default_enabled() -> i32 {
    1
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HttpApi {
    pub http_api_id: Option<i64>,
    pub http_api_name: String,
    pub http_api_path: String,
    pub http_api_desc: Option<String>,
    pub response_template: Option<String>,
    pub module: Option<String>,
    pub category: Option<String>,
    pub enabled: i32,
    /// 固定参数（JSON 对象）：执行时覆盖同名请求参数，用于锁定查询口径
    pub default_params: Option<Value>,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}

impl HttpApi {
    pub fn from_create_bo(bo: HttpApiCreateBo, user_id: i64) -> Self {
        HttpApi {
            http_api_id: None,
            http_api_name: bo.http_api_name,
            http_api_path: bo.http_api_path,
            http_api_desc: bo.http_api_desc,
            response_template: bo.response_template,
            module: bo.module,
            category: bo.category,
            enabled: bo.enabled,
            default_params: bo.default_params,
            base_entity: BaseEntity::new(user_id),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_update_bo(&mut self, bo: HttpApiUpdateBo, user_id: i64) {
        self.http_api_id = bo.http_api_id;
        self.http_api_name = bo.http_api_name;
        self.http_api_path = bo.http_api_path;
        self.http_api_desc = bo.http_api_desc;
        self.response_template = bo.response_template;
        self.module = bo.module;
        self.category = bo.category;
        self.enabled = bo.enabled;
        self.default_params = bo.default_params;
        self.base_entity.update(user_id);
    }

    pub fn to_detail_vo(&self) -> HttpApiDetailVo {
        HttpApiDetailVo {
            http_api_id: self.http_api_id.unwrap_or_default(),
            http_api_name: self.http_api_name.clone(),
            http_api_path: self.http_api_path.clone(),
            http_api_desc: self.http_api_desc.clone(),
            response_template: self.response_template.clone(),
            module: self.module.clone(),
            category: self.category.clone(),
            enabled: self.enabled,
            default_params: self.default_params.clone(),
            queries: Vec::new(),
            base_entity: BaseVo::new(self.base_entity.clone()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct HttpApiCreateBo {
    #[validate(length(min = 1, message = "http_api_name cannot be empty"))]
    pub http_api_name: String,
    #[validate(length(min = 1, message = "http_api_path cannot be empty"))]
    pub http_api_path: String,
    pub http_api_desc: Option<String>,
    pub response_template: Option<String>,
    #[serde(default = "default_enabled")]
    pub enabled: i32,
    pub module: Option<String>,
    pub category: Option<String>,
    /// 固定参数 JSON 对象（键值都会以字符串形式参与 SQL 模板渲染）
    pub default_params: Option<Value>,
    #[serde(default)]
    pub queries: Vec<HttpApiQueryCreateBo>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct HttpApiUpdateBo {
    #[validate(required)]
    pub http_api_id: Option<i64>,
    #[validate(length(min = 1, message = "http_api_name cannot be empty"))]
    pub http_api_name: String,
    #[validate(length(min = 1, message = "http_api_path cannot be empty"))]
    pub http_api_path: String,
    pub http_api_desc: Option<String>,
    pub response_template: Option<String>,
    pub enabled: i32,
    pub module: Option<String>,
    pub category: Option<String>,
    pub default_params: Option<Value>,
    #[serde(default)]
    pub queries: Vec<HttpApiQueryCreateBo>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct HttpApiListVo {
    pub http_api_id: i64,
    pub http_api_name: String,
    pub http_api_path: String,
    pub http_api_desc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    pub enabled: i32,
    #[serde(flatten)]
    pub base_entity: BaseVo,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct HttpApiDetailVo {
    pub http_api_id: i64,
    pub http_api_name: String,
    pub http_api_path: String,
    pub http_api_desc: Option<String>,
    pub response_template: Option<String>,
    pub module: Option<String>,
    pub category: Option<String>,
    pub enabled: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_params: Option<Value>,
    #[serde(default)]
    pub queries: Vec<HttpApiQueryVo>,
    #[serde(flatten)]
    pub base_entity: BaseVo,
}
