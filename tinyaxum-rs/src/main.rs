mod app;
mod auth;
mod biz;
mod cli;
mod common;
mod config;
mod error;
mod middleware;
mod sys;
mod util;

use std::path::Path;

use ::config::{Config, File};
use anyhow::Context;
use axum::{Router, middleware::from_fn_with_state};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    auth::route::auth_route::auth_route,
    biz::{
        ai_chat::route::ai_chat_route::ai_chat_route,
        ai_provider::route::ai_provider_route::ai_provider_route,
        dashboard::route::dashboard_route::dashboard_route,
        datasource::route::data_source_route::data_source_route,
        httpapi::route::http_api_route::{http_api_route, public_route},
        sqlquery::route::{
            sql_query_route::sql_query_route,
            sql_snippet_route::sql_snippet_route,
        },
        transtask::route::trans_task_route::trans_task_route,
    },
    common::handler::module_option_route::module_option_route,
    config::config::AppConfig,
    middleware::auth::auth,
    sys::user::route::user_route::user_route,
};

fn init_tracing(log_config: &crate::config::config::LogConfig) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            format!("{}=info", env!("CARGO_CRATE_NAME")).into()
        });

    let log_dir = Path::new(&log_config.dir);
    if let Err(e) = std::fs::create_dir_all(log_dir) {
        eprintln!("警告:无法创建日志目录 {:?}: {}", log_dir, e);
    }

    let file_appender =
        tracing_appender::rolling::daily(log_dir, &log_config.prefix);
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let console_layer = tracing_subscriber::fmt::layer();
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    std::mem::forget(_guard);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match cli::parse(std::env::args())? {
        cli::Cli::Serve => {}
        cli::Cli::WriteConfig { config, answers } => {
            let answers = crate::config::answers::Answers::load(&answers)?;
            crate::config::writer::write_config(&config, &answers)?;
            return Ok(());
        }
        cli::Cli::CheckConfig { config } => {
            crate::config::checker::check_config(&config).await?;
            return Ok(());
        }
    }

    let config = Config::builder()
        .add_source(File::with_name("config.toml"))
        .build()
        .context("读取 config.toml 失败")?
        .try_deserialize::<AppConfig>()
        .context("解析配置文件失败")?;

    init_tracing(&config.log);

    crate::util::metrics::init_metrics();

    let app_state = app::init_app(&config).await?;

    let protected_routes = Router::new()
        .merge(auth_route())
        .merge(user_route())
        .merge(data_source_route())
        .merge(trans_task_route())
        .merge(sql_snippet_route())
        .merge(sql_query_route())
        .merge(dashboard_route())
        .merge(http_api_route())
        .merge(public_route())
        .merge(ai_provider_route())
        .merge(ai_chat_route())
        .merge(module_option_route())
        .route_layer(from_fn_with_state(app_state.clone(), auth));

    let mut app = Router::new().nest(
        "/api",
        Router::new()
            .route("/health", axum::routing::get(|| async { "ok" }))
            .route(
                "/metrics",
                axum::routing::get(|| async {
                    let body = crate::util::metrics::metrics_output();
                    (
                        axum::http::StatusCode::OK,
                        [(
                            axum::http::header::CONTENT_TYPE,
                            "text/plain; version=0.0.4; charset=utf-8",
                        )],
                        body,
                    )
                }),
            )
            .merge(protected_routes)
            .with_state(app_state),
    );

    if let Some(dir) = &config.server.static_dir {
        if std::path::Path::new(dir).is_dir() {
            let index = std::path::Path::new(dir).join("index.html");
            app = app.fallback_service(
                tower_http::services::ServeDir::new(dir).not_found_service(
                    tower_http::services::ServeFile::new(index),
                ),
            );
            tracing::info!(static_dir = %dir, "已启用静态文件托管");
        } else {
            tracing::warn!(static_dir = %dir, "static_dir 不存在，跳过静态托管");
        }
    }

    let app = if let Some(cors) = config.cors.cors_layer() {
        tracing::info!(
            patterns = ?config.cors.allowed_origin_patterns,
            "已启用 CORS 跨域白名单"
        );
        app.layer(cors)
    } else {
        app
    };
    let app = app.layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        config.server.host, config.server.port
    ))
    .await
    .context("绑定监听地址失败")?;
    let addr = listener.local_addr().context("读取监听地址失败")?;
    tracing::info!(addr = %addr, "服务启动监听");
    axum::serve(listener, app)
        .await
        .context("启动 HTTP 服务失败")?;
    Ok(())
}
