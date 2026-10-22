use serde::{Deserialize, Serialize};

use crate::common::model::entity::BaseEntity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SqlQueryResultField {
    pub result_field_id: Option<i64>,
    pub sql_query_id: i64,
    pub field_name: String,
    pub comment_label: Option<String>,
    pub field_type: String,
    pub format_pattern: Option<String>,
    pub field_desc: Option<String>,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}
