use std::{convert::Infallible, sync::Arc};

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
};
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio_stream::{StreamExt as _, wrappers::ReceiverStream};
use validator::Validate;

use crate::{
    app::AppState,
    biz::transtask::model::trans_task::{
        TransTaskBatchCreateBo, TransTaskCreateBo, TransTaskDetailVo,
        TransTaskListVo, TransTaskUpdateBo,
    },
    common::vo::{
        page::{PageQuery, PageResult, deserialize_opt_i64_from_str},
        response::R,
    },
    error::error::AppError,
    middleware::extractors::CurrentUser,
};

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TransTaskPageQuery {
    #[serde(flatten)]
    #[validate(nested)]
    pub page: PageQuery,
    pub keyword: Option<String>,
    #[serde(default, deserialize_with = "deserialize_opt_i64_from_str")]
    pub data_source_id: Option<i64>,
    pub table_name: Option<String>,
    pub table_comment: Option<String>,
    pub task_status: Option<String>,
}

#[tracing::instrument(skip(state))]
pub async fn list_trans_task(
    State(state): State<Arc<AppState>>,
) -> Result<R<Vec<TransTaskListVo>>, AppError> {
    let data = state.services.trans_task_service.list().await?;
    Ok(R::ok_with_data(data))
}

#[tracing::instrument(skip(state, query))]
pub async fn list_trans_task_page(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TransTaskPageQuery>,
) -> Result<R<PageResult<TransTaskListVo>>, AppError> {
    let result = state.services.trans_task_service.page(query).await?;
    Ok(R::ok_with_data(result))
}

#[tracing::instrument(skip(state), fields(id = id))]
pub async fn get_trans_task_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<TransTaskDetailVo>, AppError> {
    let trans_task = state
        .services
        .trans_task_service
        .get_trans_task_by_id(id)
        .await?;
    Ok(R::ok_with_data(trans_task))
}

#[tracing::instrument(skip(state, bo), fields(username = %current_user.username))]
pub async fn create_trans_task(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<TransTaskCreateBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;
    state
        .services
        .trans_task_service
        .insert_by_bo(bo, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state, current_user, bo), fields(username = %current_user.username))]
pub async fn create_trans_task_batch(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<TransTaskBatchCreateBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;
    state
        .services
        .trans_task_service
        .insert_batch_by_bo(bo, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state, current_user, bo), fields(username = %current_user.username))]
pub async fn update_trans_task(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<TransTaskUpdateBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;
    state
        .services
        .trans_task_service
        .update_by_bo(bo, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state, current_user), fields(id = id, username = %current_user.username))]
pub async fn delete_trans_task(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .trans_task_service
        .delete_by_id(id, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state), fields(id = id))]
pub async fn true_delete_trans_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .trans_task_service
        .true_delete_by_id(id)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state), fields(id = id))]
pub async fn test_trans_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<bool>, AppError> {
    let message = state.services.trans_task_service.run_trans_task(id).await?;
    Ok(R::ok_with_data_and_message(true, message))
}

#[tracing::instrument(skip(state), fields(id = id))]
pub async fn start_trans_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .trans_task_service
        .start_trans_task(id)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state), fields(id = id))]
pub async fn stop_trans_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .trans_task_service
        .stop_trans_task(id)
        .await?;
    Ok(R::ok())
}

/// 带实时进度的同步执行端点（SSE 流）。
///
/// 响应为 `text/event-stream`，依次推送：
///   - `counting`     → 开始统计行数
///   - `totalRows`    → 总行数
///   - `exporting`    → 正在导出
///   - `merging`      → 正在合并（增量模式）
///   - `done`         → 成功完成（含最终行数和水位）
///   - `error`        → 失败详情
#[tracing::instrument(skip(state), fields(id = id))]
pub async fn run_trans_task_sse(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) =
        mpsc::channel::<crate::util::duckdb_util::SyncProgressEvent>(32);

    let service = state.services.trans_task_service.clone();
    tokio::spawn(async move {
        if let Err(e) = service.run_trans_task_with_progress(id, tx).await {
            tracing::error!(error = %e, task_id = id, "传输任务执行失败");
        }
    });

    let stream = ReceiverStream::new(rx).map(|event| {
        let data = serde_json::to_string(&event).unwrap_or_default();
        Ok::<Event, Infallible>(Event::default().event("progress").data(data))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// 传输任务完成事件的 SSE 流（供前端列表实时更新）。
///
/// 响应为 `text/event-stream`，每当任意传输任务（cron 或手动）同步完成时
/// 推送一条 `event: synced` 事件，payload 含 `taskId` 和 `rowCount`。
pub async fn transtask_events_sse(
    State(state): State<Arc<AppState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.infra.task_events_tx.subscribe();

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let data = serde_json::to_string(&event).unwrap_or_default();
                    yield Ok::<Event, Infallible>(
                        Event::default().event("synced").data(data)
                    );
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
