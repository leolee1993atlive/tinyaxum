use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use validator::Validate;

use crate::{
    app::AppState,
    biz::ai_chat::model::ai_conversation::{
        AiConversationCreateBo, AiConversationListVo, AiConversationUpdateBo,
        ListConversationsQuery,
    },
    common::vo::response::R,
    error::error::AppError,
    middleware::extractors::CurrentUser,
};

#[tracing::instrument(skip(state, query))]
pub async fn list_conversations(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListConversationsQuery>,
) -> Result<R<Vec<AiConversationListVo>>, AppError> {
    let data = state
        .services
        .ai_conversation_service
        .list_by_sql_query(query.sql_query_id)
        .await?;
    Ok(R::ok_with_data(data))
}

#[tracing::instrument(skip(state, bo), fields(username = %current_user.username))]
pub async fn create_conversation(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<AiConversationCreateBo>,
) -> Result<R<i64>, AppError> {
    bo.validate()?;
    let id = state
        .services
        .ai_conversation_service
        .create(bo, &current_user)
        .await?;
    Ok(R::ok_with_data(id))
}

#[tracing::instrument(skip(state, bo), fields(id = id, username = %current_user.username))]
pub async fn update_conversation(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(bo): Json<AiConversationUpdateBo>,
) -> Result<R<()>, AppError> {
    state
        .services
        .ai_conversation_service
        .update(id, bo, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state, current_user), fields(id = id, username = %current_user.username))]
pub async fn delete_conversation(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .ai_conversation_service
        .delete(id, &current_user)
        .await?;
    Ok(R::ok())
}
