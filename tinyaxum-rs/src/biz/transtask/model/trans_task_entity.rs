use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SeaORM entity for the `biz_trans_task` table.
#[sea_orm::model]
#[derive(
    Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "biz_trans_task")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub trans_task_id: i64,
    pub data_source_id: i64,
    pub table_name: String,
    pub table_comment: String,
    pub remark: Option<String>,
    pub row_count: i64,
    pub last_trans_time: DateTimeUtc,
    pub task_cron: String,
    pub task_status: String,
    pub task_uuid: Option<String>,
    pub sync_mode: String,
    pub sync_strategy: Option<String>,
    pub sync_column: Option<String>,
    pub sync_watermark: Option<String>,
    pub sync_primary_key: Option<String>,
    pub is_syncing: i32,
    pub version: i64,
    pub created_by: i64,
    pub created_date: DateTimeUtc,
    pub last_modified_by: i64,
    pub last_modified_date: DateTimeUtc,
    pub deleted_by: Option<i64>,
    pub deleted_date: Option<DateTimeUtc>,
}

impl ActiveModelBehavior for ActiveModel {}
