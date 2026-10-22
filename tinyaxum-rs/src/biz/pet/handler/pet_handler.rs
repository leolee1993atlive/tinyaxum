use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use validator::Validate;

use crate::{
    app::AppState,
    biz::pet::model::pet::{Pet, PetCreateBo, PetListVo, PetUpdateBo},
    common::vo::response::R,
    error::error::AppError,
    middleware::extractors::CurrentUser,
};

pub async fn list_pet(
    State(state): State<Arc<AppState>>,
) -> Result<R<Vec<PetListVo>>, AppError> {
    let result = state.services.pet_service.list().await;

    if result.is_empty() {
        Ok(R::ok_with_data(Vec::new()))
    } else {
        Ok(R::ok_with_data(result))
    }
}

pub async fn get_pet_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<Pet>, AppError> {
    let result = state
        .services
        .pet_service
        .get_pet_by_id(id)
        .await
        .ok_or(AppError::NotFound("查询失败".to_string()))?;

    Ok(R::ok_with_data(result))
}

pub async fn create_pet(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<PetCreateBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;

    let result = state
        .services
        .pet_service
        .insert_by_bo(bo, &current_user)
        .await;

    if !result {
        return Err(AppError::BusinessError("创建失败"));
    }

    Ok(R::ok())
}

pub async fn update_pet(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<PetUpdateBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;

    let result = state
        .services
        .pet_service
        .update_by_bo(bo, &current_user)
        .await;

    if !result {
        return Err(AppError::BusinessError("更新失败"));
    }

    Ok(R::ok())
}

pub async fn delete_pet(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    let result = state
        .services
        .pet_service
        .delete_by_id(id, &current_user)
        .await;

    if !result {
        return Err(AppError::BusinessError("删除失败"));
    }

    Ok(R::ok())
}

pub async fn true_delete_pet(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    let result = state
        .services
        .pet_service
        .true_delete_by_id(id)
        .await;

    if !result {
        return Err(AppError::BusinessError("删除失败"));
    }

    Ok(R::ok())
}
