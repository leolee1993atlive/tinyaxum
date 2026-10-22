use serde_json::Value;

use crate::{
    app::Infrastructure,
    biz::dashboard::{
        model::{
            dashboard_config::{
                default_dashboard_config, validate_dashboard_config_json,
            },
            dashboard_config_entity,
        },
        repository::dashboard_config_repo::DashboardConfigRepository,
    },
    error::error::AppResult,
    middleware::extractors::CurrentUser,
};

#[derive(Clone)]
pub struct DashboardConfigService {
    repo: DashboardConfigRepository,
}

impl DashboardConfigService {
    pub fn new(infra: &Infrastructure) -> Self {
        Self {
            repo: DashboardConfigRepository::new(infra.db.clone()),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_config_for_user(
        &self,
        user: &CurrentUser,
    ) -> AppResult<Value> {
        let uid = user.get_user_id();
        match self.repo.select_by_user_id(&uid).await {
            Ok(Some(row)) => {
                tracing::info!(user_id = uid, "获取看板配置成功");
                Ok(row
                    .config_json
                    .as_ref()
                    .filter(|v| !v.is_null())
                    .cloned()
                    .unwrap_or_else(default_dashboard_config))
            }
            Ok(None) => {
                tracing::info!(user_id = uid, "获取看板配置成功，使用默认配置");
                Ok(default_dashboard_config())
            }
            Err(e) => {
                tracing::error!(user_id = uid, error = ?e, "读取看板配置失败");
                Err(e)
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn save_config_for_user(
        &self,
        config: Value,
        user: &CurrentUser,
    ) -> AppResult<()> {
        validate_dashboard_config_json(&config)?;
        let uid = user.get_user_id();
        let config_json = Some(config);

        match self.repo.select_by_user_id(&uid).await {
            Ok(Some(mut existing)) => {
                existing.config_json = config_json;
                existing.version += 1;
                existing.last_modified_by = uid;
                existing.last_modified_date = chrono::Utc::now();
                self.repo.update_config_by_user_id(&uid, &existing).await?;
                tracing::info!(user_id = uid, "更新看板配置成功");
                Ok(())
            }
            Ok(None) => {
                let model = dashboard_config_entity::Model {
                    user_id: uid,
                    config_json,
                    version: 1,
                    created_by: uid,
                    created_date: chrono::Utc::now(),
                    last_modified_by: uid,
                    last_modified_date: chrono::Utc::now(),
                    deleted_by: None,
                    deleted_date: None,
                };
                self.repo.insert(&model).await?;
                tracing::info!(user_id = uid, "写入看板配置成功");
                Ok(())
            }
            Err(e) => {
                tracing::error!(user_id = uid, error = ?e, "读取看板配置失败");
                Err(e)
            }
        }
    }
}
