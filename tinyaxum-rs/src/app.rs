use std::{sync::Arc, time::Duration};

use anyhow::Context;
use deadpool_redis::Pool;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tokio::sync::broadcast;
use tokio_cron_scheduler::JobScheduler;

use crate::{
    biz::{
        ai_chat::service::{
            ai_conversation_service::AiConversationService,
            ai_message_service::AiMessageService,
        },
        ai_provider::service::ai_provider_service::AiProviderService,
        dashboard::service::{
            dashboard_config_service::DashboardConfigService,
            dashboard_query_service::DashboardQueryService,
        },
        datasource::service::data_source_service::DataSourceService,
        httpapi::service::http_api_service::HttpApiService,
        sqlquery::service::{
            sql_query_service::SqlQueryService,
            sql_snippet_service::SqlSnippetService,
        },
        transtask::service::trans_task_service::TransTaskService,
    },
    config::config::AppConfig,
    sys::user::service::user_service::UserService,
};

/// 任一传输任务同步完成后广播的事件（cron 和手动同步均发送）
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransTaskSyncedEvent {
    pub task_id: i64,
    pub data_source_id: i64,
    pub row_count: i64,
}

#[derive(Clone)]
pub struct AppState {
    pub infra: Infrastructure,
    pub services: ServiceContainer,
}

#[derive(Clone)]
pub struct Infrastructure {
    pub db: DatabaseConnection,
    pub pool: Arc<ConnectionPool>,
    pub sched: Option<Arc<JobScheduler>>,
    pub jwt_secret: String,
    pub jwt_access_ttl: u64,
    pub jwt_refresh_ttl: u64,
    pub encryption_key: String,
    pub duckdb_data_dir: std::path::PathBuf,
    pub task_events_tx: broadcast::Sender<TransTaskSyncedEvent>,
    pub dashboard_cache_ttl_secs: u64,
    pub dashboard_cache_max_rows: i64,
}

#[derive(Clone)]
pub struct ConnectionPool {
    pub redis_pool: Pool,
}

#[derive(Clone)]
pub struct ServiceContainer {
    pub user_service: UserService,
    pub data_source_service: DataSourceService,
    pub trans_task_service: TransTaskService,
    pub sql_snippet_service: SqlSnippetService,
    pub sql_query_service: SqlQueryService,
    pub dashboard_config_service: DashboardConfigService,
    pub dashboard_query_service: DashboardQueryService,
    pub http_api_service: HttpApiService,
    pub ai_provider_service: AiProviderService,
    pub ai_conversation_service: AiConversationService,
    pub ai_message_service: AiMessageService,
}

impl ServiceContainer {
    pub fn new(infra: &Infrastructure) -> anyhow::Result<Self> {
        let sql_query_service = SqlQueryService::new(infra)?;
        let ai_provider_service = AiProviderService::new(infra);
        let ai_conversation_service = AiConversationService::new(infra);
        let data_source_service = DataSourceService::new(infra);
        let dashboard_query_service =
            DashboardQueryService::new(infra, sql_query_service.clone());
        let ai_message_service = AiMessageService::new(
            infra,
            ai_provider_service.clone(),
            ai_conversation_service.clone(),
            data_source_service.clone(),
            sql_query_service.clone(),
        );
        Ok(Self {
            user_service: UserService::new(infra),
            data_source_service,
            trans_task_service: TransTaskService::new(infra),
            sql_snippet_service: SqlSnippetService::new(infra),
            sql_query_service: sql_query_service.clone(),
            dashboard_config_service: DashboardConfigService::new(infra),
            dashboard_query_service,
            http_api_service: HttpApiService::new(infra, sql_query_service),
            ai_provider_service,
            ai_conversation_service,
            ai_message_service,
        })
    }
}

#[tracing::instrument(skip(config))]
pub async fn init_app(config: &AppConfig) -> anyhow::Result<Arc<AppState>> {
    let redis_url = crate::config::dsn::build_redis_url(&config.redis);
    let mut redis_cfg = deadpool_redis::Config::from_url(redis_url.clone());
    redis_cfg.pool = Some(deadpool_redis::PoolConfig {
        max_size: config.pool.redis_max_connections,
        timeouts: deadpool_redis::Timeouts {
            wait: Some(Duration::from_secs(5)),
            create: Some(Duration::from_secs(5)),
            recycle: Some(Duration::from_secs(30)),
        },
        ..Default::default()
    });
    let redis_pool = redis_cfg
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .map_err(|e| {
            anyhow::anyhow!(
                "创建Redis连接池失败 (url={}, host={}:{}, db={}): {e:?}",
                redis_url,
                config.redis.url,
                config.redis.port,
                config.redis.db,
            )
        })?;
    tracing::info!(
        host = %config.redis.url,
        port = config.redis.port,
        db = config.redis.db,
        "成功连接 Redis"
    );

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
            tracing::info!("成功初始化 JobScheduler");

            match sched.start().await {
                Ok(_) => {
                    tracing::info!("成功启动 JobScheduler");
                }
                Err(e) => {
                    tracing::error!(error = %e, "启动 JobScheduler 失败");
                }
            }
            Some(Arc::new(sched))
        }
        Err(e) => {
            tracing::error!(error = %e, "初始化 JobScheduler 失败");
            None
        }
    };

    let duckdb_data_dir = std::path::PathBuf::from(&config.duckdb.data_dir);
    if let Err(e) = std::fs::create_dir_all(&duckdb_data_dir) {
        tracing::warn!(error = %e, path = %duckdb_data_dir.display(), "创建 DuckDB data_dir 失败");
    }

    let (task_events_tx, _) = broadcast::channel::<TransTaskSyncedEvent>(64);

    if config.encryption.key.is_empty() {
        anyhow::bail!(
            "加密密钥缺失: 请在 config.toml 的 [encryption] 部分配置 key"
        );
    }

    let infra = Infrastructure {
        db: db.clone(),
        pool: Arc::new(ConnectionPool { redis_pool }),
        sched,
        jwt_secret: config.jwt.secret.clone(),
        jwt_access_ttl: config.jwt.access_ttl as u64,
        jwt_refresh_ttl: config.jwt.refresh_ttl as u64,
        encryption_key: config.encryption.key.clone(),
        duckdb_data_dir,
        task_events_tx: task_events_tx.clone(),
        dashboard_cache_ttl_secs: config.dashboard_cache.ttl_days * 24 * 3600,
        dashboard_cache_max_rows: config.dashboard_cache.max_rows,
    };
    let services = ServiceContainer::new(&infra)?;

    {
        let mut rx = task_events_tx.subscribe();
        let dash_svc = services.dashboard_query_service.clone();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(ev) => {
                        dash_svc.bump_source_version(ev.data_source_id).await
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
    }

    // 启动时清理残留 is_syncing=1(上次进程异常退出未释放的)
    if let Err(e) = services
        .trans_task_service
        .reset_all_syncing_on_startup()
        .await
    {
        tracing::error!(error = %e, "启动时清理残留同步锁失败");
    }

    // Schedule all existing tasks
    if let Err(e) = services.trans_task_service.schedule_all_tasks().await {
        tracing::error!(error = %e, "调度已有传输任务失败");
    }

    Ok(Arc::new(AppState { infra, services }))
}
