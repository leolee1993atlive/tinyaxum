use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "biz_ai_provider")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub ai_provider_id: i64,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub system_prompt: String,
    pub temperature: Decimal,
    pub is_default: i8,
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
