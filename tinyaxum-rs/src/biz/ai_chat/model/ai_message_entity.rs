use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "biz_ai_message")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub ai_message_id: i64,
    pub ai_conversation_id: i64,
    pub role: String,
    pub content: String,
    pub attached_schema: i8,
    pub prompt_tokens: Option<i32>,
    pub completion_tokens: Option<i32>,
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
