use std::{collections::HashMap, sync::Arc};

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use validator::Validate;

use crate::{
    app::AppState,
    biz::dashboard::{
        model::{
            dashboard_config::{DashboardConfigSaveBo, DashboardConfigVo},
            dashboard_run::DashboardRunVo,
        },
        service::dashboard_query_service::DashboardQueryService,
    },
    common::vo::response::R,
    error::error::AppError,
    middleware::extractors::CurrentUser,
};

pub async fn get_dashboard_config(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
) -> Result<R<DashboardConfigVo>, AppError> {
    let config = state
        .services
        .dashboard_config_service
        .get_config_for_user(&current_user)
        .await?;
    Ok(R::ok_with_data(DashboardConfigVo { config }))
}

pub async fn save_dashboard_config(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<DashboardConfigSaveBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;
    state
        .services
        .dashboard_config_service
        .save_config_for_user(bo.config, &current_user)
        .await?;
    Ok(R::ok())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunDashboardQueryParams {
    pub month: String,
    #[serde(default)]
    pub no_cache: bool,
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

#[tracing::instrument(skip(state), fields(id = id))]
pub async fn run_dashboard_query(
    State(state): State<Arc<AppState>>,
    _current_user: CurrentUser,
    Path(id): Path<i64>,
    Query(q): Query<RunDashboardQueryParams>,
) -> Result<R<DashboardRunVo>, AppError> {
    let (start, end) = DashboardQueryService::month_date_range(&q.month)?;
    let mut params = q.extra;
    params.insert("startDate".to_string(), start);
    params.insert("endDate".to_string(), end);

    let (result, cached) = state
        .services
        .dashboard_query_service
        .run_cached(id, &q.month, params, q.no_cache)
        .await?;
    Ok(R::ok_with_data(DashboardRunVo { result, cached }))
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvalidateCacheParams {
    pub month: String,
}

#[tracing::instrument(skip(state))]
pub async fn invalidate_dashboard_cache(
    State(state): State<Arc<AppState>>,
    _current_user: CurrentUser,
    Query(q): Query<InvalidateCacheParams>,
) -> Result<R<()>, AppError> {
    state
        .services
        .dashboard_query_service
        .invalidate_month(&q.month)
        .await?;
    Ok(R::ok())
}
