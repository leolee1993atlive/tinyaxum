use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    common::model::entity::{BaseEntity, BaseVo},
    middleware::extractors::CurrentUser,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiConversation {
    pub ai_conversation_id: Option<i64>,
    pub sql_query_id: i64,
    pub ai_provider_id: i64,
    pub title: String,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}

impl AiConversation {
    pub fn new(
        sql_query_id: i64,
        ai_provider_id: i64,
        title: String,
        user: &CurrentUser,
    ) -> Self {
        Self {
            ai_conversation_id: None,
            sql_query_id,
            ai_provider_id,
            title,
            base_entity: BaseEntity::new(user.get_user_id()),
        }
    }

    pub fn to_vo(&self, provider_name: Option<String>) -> AiConversationListVo {
        AiConversationListVo {
            ai_conversation_id: self.ai_conversation_id.unwrap_or_default(),
            sql_query_id: self.sql_query_id,
            ai_provider_id: self.ai_provider_id,
            provider_name,
            title: self.title.clone(),
            base_entity: BaseVo::new(self.base_entity.clone()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AiConversationCreateBo {
    #[validate(range(min = 1))]
    pub sql_query_id: i64,
    pub ai_provider_id: Option<i64>,
    pub title: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AiConversationUpdateBo {
    pub title: Option<String>,
    pub ai_provider_id: Option<i64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConversationListVo {
    pub ai_conversation_id: i64,
    pub sql_query_id: i64,
    pub ai_provider_id: i64,
    pub provider_name: Option<String>,
    pub title: String,
    #[serde(flatten)]
    pub base_entity: BaseVo,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListConversationsQuery {
    pub sql_query_id: i64,
}
