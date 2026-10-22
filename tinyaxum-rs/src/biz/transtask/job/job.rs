use std::{path::PathBuf, pin::Pin};

use uuid::Uuid;

use crate::{
    biz::{
        datasource::model::data_source::DataSource,
        transtask::{
            model::trans_task::TransTaskDetailVo,
            repository::trans_task_repo::TransTaskRepository,
        },
    },
    common::job::job::CronJob,
    util::duckdb_util::trans_table,
};

pub struct TransTaskJob {
    pub data_source: DataSource,
    pub trans_task: TransTaskDetailVo,
    pub data_dir: PathBuf,
    pub trans_task_repo: TransTaskRepository,
    pub task_events_tx:
        tokio::sync::broadcast::Sender<crate::app::TransTaskSyncedEvent>,
}

impl CronJob for TransTaskJob {
    #[tracing::instrument(skip(self, uuid))]
    fn execute(
        &self,
        uuid: Uuid,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            tracing::info!(
                job_uuid = %uuid,
                trans_task_id = self.trans_task.trans_task_id,
                "传输任务定时作业开始执行"
            );
            tracing::info!(
                source_datasource_id = self.data_source.data_source_id,
                target_datasource = "DuckDB",
                "开始数据转换"
            );
            // trans_table 是阻塞的 DuckDB / 文件 I/O 调用,
            // 必须放到 spawn_blocking 里,避免阻塞 tokio worker 导致
            // /api/health 等接口无响应。
            let data_source = self.data_source.clone();
            let trans_task = self.trans_task.clone();
            let data_dir = self.data_dir.clone();
            let trans_task_id = trans_task.trans_task_id;

            // 抢占同步锁; 若已有手动/SSE 触发的同步在跑,则跳过本次 cron 触发
            match self
                .trans_task_repo
                .try_acquire_syncing(trans_task_id)
                .await
            {
                Ok(true) => {}
                Ok(false) => {
                    tracing::warn!(
                        trans_task_id = trans_task_id,
                        "任务已有同步在执行,跳过本次 cron 触发"
                    );
                    return;
                }
                Err(e) => {
                    tracing::error!(
                        trans_task_id = trans_task_id,
                        error = %e,
                        "获取同步锁失败,跳过本次 cron 触发"
                    );
                    return;
                }
            }

            let result = tokio::task::spawn_blocking(move || {
                trans_table(&data_source, &trans_task, &data_dir, None)
            })
            .await;

            // 无论成败必须 release
            if let Err(e) =
                self.trans_task_repo.release_syncing(trans_task_id).await
            {
                tracing::error!(
                    trans_task_id = trans_task_id,
                    error = %e,
                    "释放同步锁失败"
                );
            }

            match result {
                Ok(Ok(sync_result)) => {
                    tracing::info!(
                        trans_task_id = trans_task_id,
                        row_count = sync_result.row_count,
                        new_watermark = ?sync_result.new_watermark,
                        "数据转换成功"
                    );
                    // 根据是否有新水位值选择不同的持久化方式
                    let persist_result = if let Some(ref watermark) =
                        sync_result.new_watermark
                    {
                        self.trans_task_repo
                            .update_sync_state_by_id(
                                trans_task_id,
                                sync_result.row_count as i64,
                                watermark,
                            )
                            .await
                    } else {
                        self.trans_task_repo
                            .update_row_count_by_id(
                                trans_task_id,
                                sync_result.row_count as i64,
                            )
                            .await
                    };
                    if let Err(e) = persist_result {
                        tracing::error!(
                            trans_task_id = trans_task_id,
                            error = %e,
                            "更新任务同步状态失败"
                        );
                    }
                    let _ = self.task_events_tx.send(
                        crate::app::TransTaskSyncedEvent {
                            task_id: trans_task_id,
                            data_source_id: self
                                .data_source
                                .data_source_id
                                .unwrap_or_default(),
                            row_count: sync_result.row_count as i64,
                        },
                    );
                }
                Ok(Err(e)) => {
                    tracing::error!(
                        trans_task_id = trans_task_id,
                        error = %e,
                        "数据转换失败"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        trans_task_id = trans_task_id,
                        error = %e,
                        "数据转换线程异常"
                    );
                }
            }
        })
    }
}
