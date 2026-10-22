use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use validator::Validate;

use crate::{
    app::AppState,
    biz::ai_provider::model::ai_provider::{
        AiProviderCreateBo, AiProviderListVo, AiProviderTestResultVo,
        AiProviderUpdateBo,
    },
    common::vo::response::R,
    error::error::AppError,
    middleware::extractors::CurrentUser,
};

#[tracing::instrument(skip(state))]
pub async fn list_providers(
    State(state): State<Arc<AppState>>,
) -> Result<R<Vec<AiProviderListVo>>, AppError> {
    let data = state.services.ai_provider_service.list().await?;
    Ok(R::ok_with_data(data))
}

#[tracing::instrument(skip(state), fields(id = id))]
pub async fn get_provider(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<AiProviderListVo>, AppError> {
    let data = state.services.ai_provider_service.get_by_id(id).await?;
    Ok(R::ok_with_data(data))
}

#[tracing::instrument(skip(state, bo), fields(username = %current_user.username))]
pub async fn create_provider(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<AiProviderCreateBo>,
) -> Result<R<i64>, AppError> {
    bo.validate()?;
    let id = state
        .services
        .ai_provider_service
        .create(bo, &current_user)
        .await?;
    Ok(R::ok_with_data(id))
}

#[tracing::instrument(skip(state, bo), fields(id = id, username = %current_user.username))]
pub async fn update_provider(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(bo): Json<AiProviderUpdateBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;
    state
        .services
        .ai_provider_service
        .update(id, bo, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state, current_user), fields(id = id, username = %current_user.username))]
pub async fn delete_provider(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .ai_provider_service
        .delete(id, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state, current_user), fields(id = id, username = %current_user.username))]
pub async fn set_default_provider(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .ai_provider_service
        .set_default(id, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state), fields(id = id))]
pub async fn test_provider(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<AiProviderTestResultVo>, AppError> {
    let result = state
        .services
        .ai_provider_service
        .test_connection(id)
        .await?;
    Ok(R::ok_with_data(result))
}
