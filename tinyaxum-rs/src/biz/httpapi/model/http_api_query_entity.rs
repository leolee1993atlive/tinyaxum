use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "biz_http_api_query")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub http_api_query_id: i64,
    pub http_api_id: i64,
    pub sql_query_id: i64,
    pub query_alias: String,
    pub sort_order: i64,
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
