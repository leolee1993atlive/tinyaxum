use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::common::model::entity::BaseEntity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HttpApiQuery {
    pub http_api_query_id: Option<i64>,
    pub http_api_id: i64,
    pub sql_query_id: i64,
    pub query_alias: String,
    pub sort_order: i64,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct HttpApiQueryCreateBo {
    pub sql_query_id: i64,
    #[validate(length(min = 1, message = "query_alias cannot be empty"))]
    pub query_alias: String,
    #[serde(default)]
    pub sort_order: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct HttpApiQueryVo {
    pub query_alias: String,
    pub sql_query_id: i64,
    pub sql_query_name: String,
}
