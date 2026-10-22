use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use validator::Validate;

use crate::{
    app::AppState,
    common::vo::{
        page::{PageQuery, PageResult},
        response::R,
    },
    error::error::AppError,
    middleware::extractors::CurrentUser,
    sys::user::model::user::{
        User, UserCreateBo, UserInfo, UserListVo, UserUpdateBo,
        UserUpdatePasswordBo,
    },
};

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserPageQuery {
    #[serde(flatten)]
    #[validate(nested)]
    pub page: PageQuery,
    pub keyword: Option<String>,
    pub username: Option<String>,
    pub nick_name: Option<String>,
    pub active: Option<String>,
}

#[tracing::instrument(skip(state))]
pub async fn list_user(
    State(state): State<Arc<AppState>>,
) -> Result<R<Vec<UserListVo>>, AppError> {
    let data = state.services.user_service.list().await?;
    Ok(R::ok_with_data(data))
}

#[tracing::instrument(skip(state), fields(page = query.page.page_no, size = query.page.page_size))]
pub async fn list_user_page(
    State(state): State<Arc<AppState>>,
    Query(query): Query<UserPageQuery>,
) -> Result<R<PageResult<UserListVo>>, AppError> {
    let result = state.services.user_service.page(query).await?;
    Ok(R::ok_with_data(result))
}

#[tracing::instrument(skip(state), fields(username = %current_user.username))]
pub async fn get_user_info(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
) -> Result<R<UserInfo>, AppError> {
    let user = state
        .services
        .user_service
        .get_user_by_username(&current_user.username)
        .await?;
    Ok(R::ok_with_data(UserInfo {
        user_id: user.user_id.unwrap_or_default(),
        username: user.username,
    }))
}

#[tracing::instrument(skip(state), fields(user_id = id))]
pub async fn get_user_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<User>, AppError> {
    let user = state.services.user_service.get_user_by_id(id).await?;
    Ok(R::ok_with_data(user))
}

#[tracing::instrument(
    skip(state, bo),
    fields(username = %current_user.username, new_username = ?bo.username)
)]
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<UserCreateBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;
    state
        .services
        .user_service
        .insert_by_bo(bo, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(
    skip(state, bo),
    fields(operator = %current_user.username, target_user_id = ?bo.user_id)
)]
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<UserUpdateBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;
    state
        .services
        .user_service
        .update_by_bo(bo, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(
    skip(state),
    fields(operator = %current_user.username, user_id = id)
)]
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .user_service
        .delete_by_id(id, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state), fields(user_id = id))]
pub async fn true_delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state.services.user_service.true_delete_by_id(id).await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state, bo), fields(user_id = ?bo.user_id))]
pub async fn update_password(
    State(state): State<Arc<AppState>>,
    Json(bo): Json<UserUpdatePasswordBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;
    let user_id = bo
        .user_id
        .ok_or_else(|| AppError::BadRequest("用户ID为空".to_string()))?;
    state
        .services
        .user_service
        .update_password(user_id, bo.password)
        .await?;
    Ok(R::ok())
}
