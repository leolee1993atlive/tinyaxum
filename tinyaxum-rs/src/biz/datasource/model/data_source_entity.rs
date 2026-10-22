use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// SeaORM entity for the `biz_data_source` table.
#[sea_orm::model]
#[derive(
    Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "biz_data_source")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub data_source_id: i64,
    pub code: String,
    pub name: String,
    pub remark: Option<String>,
    pub db_type: String,
    pub db_host: String,
    pub db_port: i32,
    pub db_name: String,
    pub db_schema: Option<String>,
    pub db_username: String,
    pub db_password: String,
    pub version: i64,
    pub created_by: i64,
    pub created_date: DateTimeUtc,
    pub last_modified_by: i64,
    pub last_modified_date: DateTimeUtc,
    pub deleted_by: Option<i64>,
    pub deleted_date: Option<DateTimeUtc>,
}

impl ActiveModelBehavior for ActiveModel {}
