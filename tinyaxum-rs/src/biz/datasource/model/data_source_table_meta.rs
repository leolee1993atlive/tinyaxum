use serde::{Deserialize, Serialize};

use crate::common::model::entity::BaseEntity;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataSourceTableMeta {
    pub table_meta_id: Option<i64>,
    pub data_source_id: i64,
    pub table_name: String,
    pub comment_label: Option<String>,
    pub table_desc: Option<String>,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}
