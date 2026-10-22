use std::{collections::HashMap, sync::Arc};

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use validator::Validate;

use crate::{
    app::AppState,
    biz::httpapi::model::http_api::{
        HttpApiCreateBo, HttpApiDetailVo, HttpApiListVo, HttpApiUpdateBo,
    },
    common::vo::{
        page::{PageQuery, PageResult, normalize_paging},
        response::R,
    },
    error::error::AppError,
    middleware::extractors::CurrentUser,
};

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct HttpApiPageQuery {
    #[serde(flatten)]
    #[validate(nested)]
    pub page: PageQuery,
    pub keyword: Option<String>,
    pub enabled: Option<i32>,
    pub module: Option<String>,
    pub category: Option<String>,
}

pub async fn list_http_api(
    State(state): State<Arc<AppState>>,
) -> Result<R<Vec<HttpApiListVo>>, AppError> {
    let page = state
        .services
        .http_api_service
        .get_page(None, 1, 1000, None, None, None)
        .await?;
    Ok(R::ok_with_data(page.records))
}

pub async fn get_http_api_page(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HttpApiPageQuery>,
) -> Result<R<PageResult<HttpApiListVo>>, AppError> {
    let (current, size, _) = normalize_paging(&query.page);
    let keyword = query.keyword.filter(|s| !s.trim().is_empty());
    let page = state
        .services
        .http_api_service
        .get_page(
            keyword,
            current,
            size,
            query.enabled,
            query.module,
            query.category,
        )
        .await?;
    Ok(R::ok_with_data(page))
}

pub async fn get_http_api_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<HttpApiDetailVo>, AppError> {
    let vo = state.services.http_api_service.get_by_id(id).await?;
    Ok(R::ok_with_data(vo))
}

pub async fn create_http_api(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<HttpApiCreateBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;
    state
        .services
        .http_api_service
        .create(bo, current_user.get_user_id())
        .await?;
    Ok(R::ok())
}

pub async fn update_http_api(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<HttpApiUpdateBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;
    state
        .services
        .http_api_service
        .update(bo, current_user.get_user_id())
        .await?;
    Ok(R::ok())
}

pub async fn delete_http_api(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .http_api_service
        .delete(id, current_user.get_user_id())
        .await?;
    Ok(R::ok())
}

pub async fn enable_http_api(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .http_api_service
        .enable(id, current_user.get_user_id())
        .await?;
    Ok(R::ok())
}

pub async fn disable_http_api(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .http_api_service
        .disable(id, current_user.get_user_id())
        .await?;
    Ok(R::ok())
}

pub async fn execute_public_api(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<R<serde_json::Value>, AppError> {
    let value = state
        .services
        .http_api_service
        .execute_public_api(&path, params)
        .await?;
    Ok(R::ok_with_data(value))
}
