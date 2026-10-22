use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    response::sse::{KeepAlive, Sse},
};
use validator::Validate;

use crate::{
    app::AppState,
    biz::ai_chat::model::ai_message::{AiMessageVo, SendMessageBo},
    common::vo::response::R,
    error::error::AppError,
    middleware::extractors::CurrentUser,
};

#[tracing::instrument(skip(state), fields(id = id))]
pub async fn list_messages(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<Vec<AiMessageVo>>, AppError> {
    let msgs = state.services.ai_message_service.list_messages(id).await?;
    let vos: Vec<AiMessageVo> = msgs.iter().map(|m| m.to_vo()).collect();
    Ok(R::ok_with_data(vos))
}

#[tracing::instrument(skip(state, bo), fields(id = id, username = %current_user.username))]
pub async fn send_message(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(bo): Json<SendMessageBo>,
) -> Result<
    Sse<
        impl futures_util::Stream<
            Item = Result<axum::response::sse::Event, std::convert::Infallible>,
        >,
    >,
    AppError,
> {
    bo.validate()?;
    let svc = state.services.ai_message_service.clone();
    let stream = svc.send_message_stream(id, bo, current_user);
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
