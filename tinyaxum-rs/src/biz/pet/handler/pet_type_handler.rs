use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use validator::Validate;

use crate::{
    app::AppState,
    biz::pet::model::pet_type::{
        PetType, PetTypeCreateBo, PetTypeListVo, PetTypeUpdateBo,
    },
    common::vo::response::R,
    error::error::AppError,
};

pub async fn list_pet_type(
    State(state): State<Arc<AppState>>,
) -> Result<R<Vec<PetTypeListVo>>, AppError> {
    let result = state.services.pet_type_service.list().await;

    if result.is_empty() {
        Ok(R::ok_with_data(Vec::new()))
    } else {
        Ok(R::ok_with_data(result))
    }
}

pub async fn get_pet_type_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<PetType>, AppError> {
    let result = state
        .services
        .pet_type_service
        .get_pet_type_by_id(id)
        .await
        .ok_or(AppError::NotFound("查询失败".to_string()))?;

    Ok(R::ok_with_data(result))
}

pub async fn create_pet_type(
    State(state): State<Arc<AppState>>,
    Json(bo): Json<PetTypeCreateBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;

    let result = state.services.pet_type_service.insert_by_bo(bo).await;

    if !result {
        return Err(AppError::BusinessError("创建失败"));
    }

    Ok(R::ok())
}

pub async fn update_pet_type(
    State(state): State<Arc<AppState>>,
    Json(bo): Json<PetTypeUpdateBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;

    let result = state.services.pet_type_service.update_by_bo(bo).await;

    if !result {
        return Err(AppError::BusinessError("更新失败"));
    }

    Ok(R::ok())
}

pub async fn delete_pet_type(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    let result = state.services.pet_type_service.delete_by_id(id).await;

    if !result {
        return Err(AppError::BusinessError("删除失败"));
    }

    Ok(R::ok())
}
