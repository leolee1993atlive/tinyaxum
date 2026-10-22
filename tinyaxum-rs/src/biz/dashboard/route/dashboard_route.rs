use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post, put},
};

use crate::{
    app::AppState,
    biz::dashboard::handler::dashboard_handler::{
        get_dashboard_config, invalidate_dashboard_cache, run_dashboard_query,
        save_dashboard_config,
    },
};

pub fn dashboard_route() -> Router<Arc<AppState>> {
    Router::new()
        .route("/dashboard/config", get(get_dashboard_config))
        .route("/dashboard/config", put(save_dashboard_config))
        .route("/dashboard/run/{id}", get(run_dashboard_query))
        .route(
            "/dashboard/cache/invalidate",
            post(invalidate_dashboard_cache),
        )
}
