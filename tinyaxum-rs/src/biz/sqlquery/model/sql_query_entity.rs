use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SeaORM entity for the `biz_sql_query` table.
#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "biz_sql_query")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub sql_query_id: i64,
    pub sql_query_name: String,
    pub sql_query_desc: Option<String>,
    pub sql_query_content: String,
    pub data_source_id: Option<i64>,
    pub result_meta_updated_at: Option<DateTimeUtc>,
    pub is_collection: i8,
    pub viz_config: Option<serde_json::Value>,
    pub module: Option<String>,
    pub category: Option<String>,
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
