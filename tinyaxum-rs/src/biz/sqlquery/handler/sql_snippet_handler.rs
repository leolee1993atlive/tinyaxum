use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use validator::Validate;

use crate::{
    app::AppState,
    biz::sqlquery::model::sql_snippet::{
        SqlSnippetCreateBo, SqlSnippetDetailVo, SqlSnippetListVo,
        SqlSnippetUpdateBo,
    },
    common::vo::{
        page::{PageQuery, PageResult},
        response::R,
    },
    error::error::AppError,
    middleware::extractors::CurrentUser,
};

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SqlSnippetPageQuery {
    #[serde(flatten)]
    #[validate(nested)]
    pub page: PageQuery,
    pub keyword: Option<String>,
    pub sql_snippet_name: Option<String>,
    pub tag: Option<String>,
}

#[tracing::instrument(skip(state))]
pub async fn list_sql_snippet(
    State(state): State<Arc<AppState>>,
) -> Result<R<Vec<SqlSnippetListVo>>, AppError> {
    let data = state.services.sql_snippet_service.list().await?;
    Ok(R::ok_with_data(data))
}

#[tracing::instrument(skip(state, query))]
pub async fn list_sql_snippet_page(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SqlSnippetPageQuery>,
) -> Result<R<PageResult<SqlSnippetListVo>>, AppError> {
    let result = state.services.sql_snippet_service.page(query).await?;
    Ok(R::ok_with_data(result))
}

#[tracing::instrument(skip(state), fields(id = id))]
pub async fn get_sql_snippet_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<SqlSnippetDetailVo>, AppError> {
    let sql_snippet = state
        .services
        .sql_snippet_service
        .get_sql_snippet_by_id(id)
        .await?;
    Ok(R::ok_with_data(sql_snippet))
}

#[tracing::instrument(skip(state, bo), fields(username = %current_user.username))]
pub async fn create_sql_snippet(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<SqlSnippetCreateBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;
    state
        .services
        .sql_snippet_service
        .insert_by_bo(bo, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state, bo), fields(username = %current_user.username))]
pub async fn update_sql_snippet(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<SqlSnippetUpdateBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;
    state
        .services
        .sql_snippet_service
        .update_by_bo(bo, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state, current_user), fields(id = id, username = %current_user.username))]
pub async fn delete_sql_snippet(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .sql_snippet_service
        .delete_by_id(id, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state), fields(id = id))]
pub async fn true_delete_sql_snippet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .sql_snippet_service
        .true_delete_by_id(id)
        .await?;
    Ok(R::ok())
}
