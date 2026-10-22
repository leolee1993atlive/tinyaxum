use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    common::model::entity::{BaseEntity, BaseVo},
    middleware::extractors::CurrentUser,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiMessage {
    pub ai_message_id: Option<i64>,
    pub ai_conversation_id: i64,
    pub role: String,
    pub content: String,
    pub attached_schema: i8,
    pub prompt_tokens: Option<i32>,
    pub completion_tokens: Option<i32>,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}

impl AiMessage {
    pub fn user_msg(
        conversation_id: i64,
        content: String,
        attached_schema: bool,
        user: &CurrentUser,
    ) -> Self {
        Self {
            ai_message_id: None,
            ai_conversation_id: conversation_id,
            role: "user".to_string(),
            content,
            attached_schema: if attached_schema { 1 } else { 0 },
            prompt_tokens: None,
            completion_tokens: None,
            base_entity: BaseEntity::new(user.get_user_id()),
        }
    }

    pub fn assistant_msg(
        conversation_id: i64,
        content: String,
        prompt_tokens: Option<i32>,
        completion_tokens: Option<i32>,
        user: &CurrentUser,
    ) -> Self {
        Self {
            ai_message_id: None,
            ai_conversation_id: conversation_id,
            role: "assistant".to_string(),
            content,
            attached_schema: 0,
            prompt_tokens,
            completion_tokens,
            base_entity: BaseEntity::new(user.get_user_id()),
        }
    }

    pub fn to_vo(&self) -> AiMessageVo {
        AiMessageVo {
            ai_message_id: self.ai_message_id.unwrap_or_default(),
            ai_conversation_id: self.ai_conversation_id,
            role: self.role.clone(),
            content: self.content.clone(),
            attached_schema: self.attached_schema,
            prompt_tokens: self.prompt_tokens,
            completion_tokens: self.completion_tokens,
            base_entity: BaseVo::new(self.base_entity.clone()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageBo {
    #[validate(length(min = 1))]
    pub content: String,
    #[serde(default)]
    pub attach_schema: bool,
    pub current_sql: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiMessageVo {
    pub ai_message_id: i64,
    pub ai_conversation_id: i64,
    pub role: String,
    pub content: String,
    pub attached_schema: i8,
    pub prompt_tokens: Option<i32>,
    pub completion_tokens: Option<i32>,
    #[serde(flatten)]
    pub base_entity: BaseVo,
}
