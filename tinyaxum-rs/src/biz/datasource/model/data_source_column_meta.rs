use serde::{Deserialize, Serialize};

use crate::common::model::entity::BaseEntity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataSourceColumnMeta {
    pub column_meta_id: Option<i64>,
    pub table_meta_id: i64,
    pub column_name: String,
    pub comment_label: Option<String>,
    pub field_type: String,
    pub format_pattern: Option<String>,
    pub field_desc: Option<String>,
    pub source_physical_type: Option<String>,
    pub ordinal_position: i32,
    /// 是否为主键列：1 是，0 否
    pub is_primary_key: i32,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}
