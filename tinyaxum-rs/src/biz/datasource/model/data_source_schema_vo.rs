use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceSchemaVo {
    pub data_source_id: i64,
    pub tables: Vec<DataSourceTableSchemaVo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceTableSchemaVo {
    pub table_meta_id: i64,
    pub table_name: String,
    pub comment_label: Option<String>,
    pub table_desc: Option<String>,
    pub columns: Vec<DataSourceColumnSchemaVo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceColumnSchemaVo {
    pub column_meta_id: i64,
    pub column_name: String,
    pub comment_label: Option<String>,
    pub field_type: String,
    pub format_pattern: Option<String>,
    pub field_desc: Option<String>,
    pub source_physical_type: Option<String>,
    pub ordinal_position: i32,
    pub is_primary_key: i32,
}
