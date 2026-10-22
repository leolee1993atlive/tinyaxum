use crate::{
    app::Infrastructure,
    biz::ai_provider::{
        model::ai_provider::{
            AiProvider, AiProviderCreateBo, AiProviderListVo,
            AiProviderUpdateBo, mask_api_key,
        },
        repository::ai_provider_repo::AiProviderRepository,
    },
    error::error::{AppError, AppResult},
    middleware::extractors::CurrentUser,
    util::crypto_util::{decrypt_password, encrypt_password},
};

#[derive(Clone)]
pub struct AiProviderService {
    repo: AiProviderRepository,
    encryption_key: String,
}

impl AiProviderService {
    pub fn new(infra: &Infrastructure) -> Self {
        Self {
            repo: AiProviderRepository::new(infra.db.clone()),
            encryption_key: infra.encryption_key.clone(),
        }
    }

    fn key_bytes(&self) -> AppResult<Vec<u8>> {
        Ok(self.encryption_key.as_bytes().to_vec())
    }

    pub async fn list(&self) -> AppResult<Vec<AiProviderListVo>> {
        let providers = self.repo.select_all().await?;
        let key = self.key_bytes()?;
        let mut out = Vec::with_capacity(providers.len());
        for p in providers.iter() {
            let plain = decrypt_password(&key, &p.api_key)
                .unwrap_or_else(|_| "(解密失败)".to_string());
            out.push(p.to_list_vo(mask_api_key(&plain)));
        }
        Ok(out)
    }

    pub async fn get_by_id(&self, id: i64) -> AppResult<AiProviderListVo> {
        let p = self.repo.select_by_id(id).await?.ok_or_else(|| {
            AppError::NotFound(format!("AI provider {} 不存在", id))
        })?;
        let key = self.key_bytes()?;
        let plain = decrypt_password(&key, &p.api_key)
            .unwrap_or_else(|_| "(解密失败)".to_string());
        Ok(p.to_list_vo(mask_api_key(&plain)))
    }

    pub async fn create(
        &self,
        bo: AiProviderCreateBo,
        user: &CurrentUser,
    ) -> AppResult<i64> {
        let key = self.key_bytes()?;
        let encrypted = encrypt_password(&key, &bo.api_key)?;
        let domain = AiProvider::from_create_bo(bo, encrypted, user);
        let id = self.repo.insert(&domain).await?;
        Ok(id)
    }

    pub async fn update(
        &self,
        id: i64,
        bo: AiProviderUpdateBo,
        user: &CurrentUser,
    ) -> AppResult<()> {
        let mut existing =
            self.repo.select_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("AI provider {} 不存在", id))
            })?;
        let key = self.key_bytes()?;
        let new_encrypted = if bo.api_key.is_empty() {
            None
        } else {
            Some(encrypt_password(&key, &bo.api_key)?)
        };
        existing.apply_update(bo, new_encrypted, user);
        self.repo.update_by_id(&existing).await?;
        Ok(())
    }

    pub async fn delete(&self, id: i64, user: &CurrentUser) -> AppResult<()> {
        let mut existing =
            self.repo.select_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("AI provider {} 不存在", id))
            })?;
        existing.base_entity.delete(user.get_user_id());
        self.repo.update_by_id(&existing).await?;
        Ok(())
    }

    pub async fn set_default(
        &self,
        id: i64,
        user: &CurrentUser,
    ) -> AppResult<()> {
        let _ = self.repo.select_by_id(id).await?.ok_or_else(|| {
            AppError::NotFound(format!("AI provider {} 不存在", id))
        })?;
        self.repo
            .set_default_tx(id, user.get_user_id())
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    pub async fn get_decrypted_for_call(
        &self,
        id: i64,
    ) -> AppResult<(AiProvider, String)> {
        let p = self.repo.select_by_id_with_deleted(id).await?.ok_or_else(
            || AppError::NotFound(format!("AI provider {} 不存在", id)),
        )?;
        if p.base_entity.deleted_by.is_some() {
            return Err(AppError::BadRequest(format!(
                "PROVIDER_DELETED: AI provider {} 已被删除,请重选",
                id
            )));
        }
        let key = self.key_bytes()?;
        let plain = decrypt_password(&key, &p.api_key)?;
        Ok((p, plain))
    }

    pub async fn test_connection(
        &self,
        id: i64,
    ) -> AppResult<
        crate::biz::ai_provider::model::ai_provider::AiProviderTestResultVo,
    > {
        use std::time::Instant;

        use crate::{
            biz::ai_provider::model::ai_provider::AiProviderTestResultVo,
            util::ai_client::{ChatMessage, chat_completion_once},
        };

        let p = self.repo.select_by_id(id).await?.ok_or_else(|| {
            AppError::NotFound(format!("AI provider {} 不存在", id))
        })?;
        let key = self.key_bytes()?;
        let plain = decrypt_password(&key, &p.api_key)?;

        let start = Instant::now();
        let res = chat_completion_once(
            &p.base_url,
            &plain,
            &p.model,
            vec![
                ChatMessage {
                    role: "system".into(),
                    content: "you are a helpful assistant".into(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: "ping".into(),
                },
            ],
            p.temperature,
        )
        .await;
        let latency_ms = start.elapsed().as_millis() as u64;

        match res {
            Ok(once) => Ok(AiProviderTestResultVo {
                ok: true,
                latency_ms,
                model_echo: Some(once.content.chars().take(80).collect()),
                error: None,
            }),
            Err(e) => Ok(AiProviderTestResultVo {
                ok: false,
                latency_ms,
                model_echo: None,
                error: Some(e.to_string()),
            }),
        }
    }
}
