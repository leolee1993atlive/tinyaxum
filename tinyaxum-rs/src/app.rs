use std::{sync::Arc, time::Duration};

use anyhow::Context;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tokio_cron_scheduler::JobScheduler;

use crate::{
    config::config::AppConfig, sys::user::service::user_service::UserService,
};

#[derive(Clone)]
pub struct AppState {
    pub infra: Infrastructure,
    pub services: ServiceContainer,
}

#[derive(Clone)]
pub struct Infrastructure {
    pub db: DatabaseConnection,
    pub sched: Option<Arc<JobScheduler>>,
    pub jwt_secret: String,
    pub jwt_access_ttl: u64,
    pub jwt_refresh_ttl: u64,
    pub encryption_key: String,
}

#[derive(Clone)]
pub struct ServiceContainer {
    pub user_service: UserService,
}

impl ServiceContainer {
    pub fn new(infra: &Infrastructure) -> anyhow::Result<Self> {
        Ok(Self {
            user_service: UserService::new(infra),
        })
    }
}

#[tracing::instrument(skip(config))]
pub async fn init_app(config: &AppConfig) -> anyhow::Result<Arc<AppState>> {
    let dsn = crate::config::dsn::build_db_dsn(&config.db)?;

    let mut opt = ConnectOptions::new(&dsn);
    opt.max_connections(config.pool.db_max_connections)
        .min_connections(config.pool.db_min_connections)
        .connect_timeout(Duration::from_secs(30))
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(600))
        .sqlx_logging(false);

    let db = Database::connect(opt)
        .await
        .with_context(|| format!("数据库连接失败: {}", config.db.db_name))?;

    tracing::info!(
        db_type = %config.db.db_type,
        db_name = %config.db.db_name,
        "成功连接数据库"
    );

    let sched = match JobScheduler::new().await {
        Ok(sched) => {
            if let Err(e) = sched.start().await {
                tracing::error!(error = %e, "启动 JobScheduler 失败");
            }
            Some(Arc::new(sched))
        }
        Err(e) => {
            tracing::error!(error = %e, "初始化 JobScheduler 失败");
            None
        }
    };

    if config.encryption.key.is_empty() {
        anyhow::bail!(
            "加密密钥缺失: 请在 config.toml 的 [encryption] 部分配置 key"
        );
    }

    let infra = Infrastructure {
        db,
        sched,
        jwt_secret: config.jwt.secret.clone(),
        jwt_access_ttl: config.jwt.access_ttl as u64,
        jwt_refresh_ttl: config.jwt.refresh_ttl as u64,
        encryption_key: config.encryption.key.clone(),
    };
    let services = ServiceContainer::new(&infra)?;

    // 启动期自检：确认 [encryption] key 能完成一次 AES-256-GCM 往返
    let probe = crate::util::crypto_util::encrypt_password(
        infra.encryption_key.as_bytes(),
        "tinyaxum",
    )?;
    let roundtrip = crate::util::crypto_util::decrypt_password(
        infra.encryption_key.as_bytes(),
        &probe,
    )?;
    if roundtrip != "tinyaxum" {
        anyhow::bail!("加密密钥自检失败: [encryption] key 无法完成加解密往返");
    }
    tracing::info!(
        scheduler_ready = infra.sched.is_some(),
        "基础设施初始化完成"
    );

    Ok(Arc::new(AppState { infra, services }))
}
