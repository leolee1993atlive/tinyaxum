use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize,
)]
#[sea_orm(table_name = "biz_data_source_column_meta")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub column_meta_id: i64,
    pub table_meta_id: i64,
    pub column_name: String,
    pub comment_label: Option<String>,
    pub field_type: String,
    pub format_pattern: Option<String>,
    pub field_desc: Option<String>,
    pub source_physical_type: Option<String>,
    pub ordinal_position: i32,
    pub is_primary_key: i32,
    pub version: i64,
    pub created_by: i64,
    pub created_date: DateTimeUtc,
    pub last_modified_by: i64,
    pub last_modified_date: DateTimeUtc,
    pub deleted_by: Option<i64>,
    pub deleted_date: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
