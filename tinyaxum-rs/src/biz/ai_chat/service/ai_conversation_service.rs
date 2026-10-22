use crate::{
    app::Infrastructure,
    biz::{
        ai_chat::{
            model::ai_conversation::{
                AiConversation, AiConversationCreateBo, AiConversationListVo,
                AiConversationUpdateBo,
            },
            repository::ai_conversation_repo::AiConversationRepository,
        },
        ai_provider::repository::ai_provider_repo::AiProviderRepository,
    },
    error::error::{AppError, AppResult},
    middleware::extractors::CurrentUser,
};

#[derive(Clone)]
pub struct AiConversationService {
    repo: AiConversationRepository,
    provider_repo: AiProviderRepository,
}

impl AiConversationService {
    pub fn new(infra: &Infrastructure) -> Self {
        Self {
            repo: AiConversationRepository::new(infra.db.clone()),
            provider_repo: AiProviderRepository::new(infra.db.clone()),
        }
    }

    pub async fn list_by_sql_query(
        &self,
        sql_query_id: i64,
    ) -> AppResult<Vec<AiConversationListVo>> {
        let convs = self.repo.select_by_sql_query(sql_query_id).await?;
        let providers = self.provider_repo.select_all().await?;
        let mut out = Vec::with_capacity(convs.len());
        for c in convs.iter() {
            let pname = providers
                .iter()
                .find(|p| p.ai_provider_id == Some(c.ai_provider_id))
                .map(|p| p.name.clone());
            out.push(c.to_vo(pname));
        }
        Ok(out)
    }

    pub async fn create(
        &self,
        bo: AiConversationCreateBo,
        user: &CurrentUser,
    ) -> AppResult<i64> {
        let provider_id = match bo.ai_provider_id {
            Some(id) => id,
            None => self
                .provider_repo
                .select_default()
                .await?
                .and_then(|p| p.ai_provider_id)
                .ok_or_else(|| {
                    AppError::BadRequest(
                        "未指定 providerId 且没有默认 provider, 请先在设置中添加并设为默认".into(),
                    )
                })?,
        };
        let title = bo.title.unwrap_or_else(|| "新会话".to_string());
        let conv =
            AiConversation::new(bo.sql_query_id, provider_id, title, user);
        let id = self.repo.insert(&conv).await?;
        Ok(id)
    }

    pub async fn update(
        &self,
        id: i64,
        bo: AiConversationUpdateBo,
        user: &CurrentUser,
    ) -> AppResult<()> {
        let mut existing =
            self.repo.select_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("AI conversation {} 不存在", id))
            })?;
        if let Some(t) = bo.title {
            existing.title = t;
        }
        if let Some(pid) = bo.ai_provider_id {
            existing.ai_provider_id = pid;
        }
        existing.base_entity.update(user.get_user_id());
        self.repo.update_by_id(&existing).await?;
        Ok(())
    }

    pub async fn delete(&self, id: i64, user: &CurrentUser) -> AppResult<()> {
        let mut existing =
            self.repo.select_by_id(id).await?.ok_or_else(|| {
                AppError::NotFound(format!("AI conversation {} 不存在", id))
            })?;
        existing.base_entity.delete(user.get_user_id());
        self.repo.update_by_id(&existing).await?;
        Ok(())
    }

    pub async fn get_by_id(&self, id: i64) -> AppResult<AiConversation> {
        self.repo.select_by_id(id).await?.ok_or_else(|| {
            AppError::NotFound(format!("AI conversation {} 不存在", id))
        })
    }
}
