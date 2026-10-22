use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    common::model::entity::{BaseEntity, BaseVo},
    middleware::extractors::CurrentUser,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiProvider {
    pub ai_provider_id: Option<i64>,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub system_prompt: String,
    pub temperature: f32,
    pub is_default: i8,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}

impl AiProvider {
    pub fn from_create_bo(
        bo: AiProviderCreateBo,
        encrypted_key: String,
        user: &CurrentUser,
    ) -> Self {
        Self {
            ai_provider_id: None,
            name: bo.name,
            base_url: bo.base_url,
            api_key: encrypted_key,
            model: bo.model,
            system_prompt: bo.system_prompt,
            temperature: bo.temperature.unwrap_or(0.3),
            is_default: 0,
            base_entity: BaseEntity::new(user.get_user_id()),
        }
    }

    pub fn apply_update(
        &mut self,
        bo: AiProviderUpdateBo,
        new_encrypted_key: Option<String>,
        user: &CurrentUser,
    ) {
        self.name = bo.name;
        self.base_url = bo.base_url;
        if let Some(k) = new_encrypted_key {
            self.api_key = k;
        }
        self.model = bo.model;
        self.system_prompt = bo.system_prompt;
        if let Some(t) = bo.temperature {
            self.temperature = t;
        }
        self.base_entity.update(user.get_user_id());
    }

    pub fn to_list_vo(&self, masked_key: String) -> AiProviderListVo {
        AiProviderListVo {
            ai_provider_id: self.ai_provider_id.unwrap_or_default(),
            name: self.name.clone(),
            base_url: self.base_url.clone(),
            api_key_masked: masked_key,
            model: self.model.clone(),
            system_prompt: self.system_prompt.clone(),
            temperature: self.temperature,
            is_default: self.is_default,
            base_entity: BaseVo::new(self.base_entity.clone()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderCreateBo {
    #[validate(length(min = 1, max = 64))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub base_url: String,
    #[validate(length(min = 1, max = 512))]
    pub api_key: String,
    #[validate(length(min = 1, max = 128))]
    pub model: String,
    pub system_prompt: String,
    #[validate(range(min = 0.0, max = 2.0))]
    pub temperature: Option<f32>,
}

#[derive(Clone, Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderUpdateBo {
    #[validate(length(min = 1, max = 64))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[validate(length(min = 1, max = 128))]
    pub model: String,
    pub system_prompt: String,
    #[validate(range(min = 0.0, max = 2.0))]
    pub temperature: Option<f32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderListVo {
    pub ai_provider_id: i64,
    pub name: String,
    pub base_url: String,
    pub api_key_masked: String,
    pub model: String,
    pub system_prompt: String,
    pub temperature: f32,
    pub is_default: i8,
    #[serde(flatten)]
    pub base_entity: BaseVo,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderTestResultVo {
    pub ok: bool,
    pub latency_ms: u64,
    pub model_echo: Option<String>,
    pub error: Option<String>,
}

pub fn mask_api_key(plain: &str) -> String {
    let n = plain.chars().count();
    if n <= 4 {
        return "***".to_string();
    }
    let tail: String = plain.chars().skip(n.saturating_sub(4)).collect();
    format!("***{}", tail)
}
