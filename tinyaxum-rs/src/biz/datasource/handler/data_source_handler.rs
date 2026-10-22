use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use validator::Validate;

use crate::{
    app::AppState,
    biz::datasource::model::{
        data_source::{
            DataSourceCreateBo, DataSourceDetailVo, DataSourceListVo,
            DataSourceUpdateBo,
        },
        data_source_schema_vo::DataSourceSchemaVo,
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
pub struct DataSourcePageQuery {
    #[serde(flatten)]
    #[validate(nested)]
    pub page: PageQuery,
    pub keyword: Option<String>,
    pub code: Option<String>,
    pub name: Option<String>,
    pub db_type: Option<String>,
}

#[tracing::instrument(skip(state))]
pub async fn list_data_source(
    State(state): State<Arc<AppState>>,
) -> Result<R<Vec<DataSourceListVo>>, AppError> {
    let data = state.services.data_source_service.list().await?;
    Ok(R::ok_with_data(data))
}

#[tracing::instrument(skip(state, query))]
pub async fn list_data_source_page(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DataSourcePageQuery>,
) -> Result<R<PageResult<DataSourceListVo>>, AppError> {
    let result = state.services.data_source_service.page(query).await?;
    Ok(R::ok_with_data(result))
}

#[tracing::instrument(skip(state), fields(id = id))]
pub async fn get_data_source_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<DataSourceDetailVo>, AppError> {
    let data_source = state
        .services
        .data_source_service
        .get_data_source_by_id(id)
        .await?;
    Ok(R::ok_with_data(data_source))
}

#[tracing::instrument(skip(state, bo, current_user), fields(username = %current_user.username))]
pub async fn create_data_source(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<DataSourceCreateBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;
    state
        .services
        .data_source_service
        .insert_by_bo(bo, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state, bo, current_user), fields(username = %current_user.username))]
pub async fn update_data_source(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<DataSourceUpdateBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;
    state
        .services
        .data_source_service
        .update_by_bo(bo, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state, current_user), fields(id = id, username = %current_user.username))]
pub async fn delete_data_source(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .data_source_service
        .delete_by_id(id, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state), fields(id = id))]
pub async fn true_delete_data_source(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .data_source_service
        .true_delete_by_id(id)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state), fields(id = id))]
pub async fn test_data_source(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<bool>, AppError> {
    let message = state
        .services
        .data_source_service
        .test_data_source(id)
        .await?;
    Ok(R::ok_with_data_and_message(true, message))
}

#[tracing::instrument(skip(state), fields(id = id))]
pub async fn get_data_source_schema(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<DataSourceSchemaVo>, AppError> {
    let schema = state.services.data_source_service.get_schema(id).await?;
    Ok(R::ok_with_data(schema))
}

#[tracing::instrument(skip(state, current_user), fields(id = id, username = %current_user.username))]
pub async fn refresh_data_source_schema(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .data_source_service
        .refresh_schema(id, &current_user)
        .await?;
    Ok(R::ok())
}
