use std::sync::Arc;

use tokio_cron_scheduler::JobScheduler;
use uuid::Uuid;

use crate::{
    app::Infrastructure,
    biz::{
        datasource::{
            model::data_source::DataSource,
            repository::{
                data_source_column_meta_repo::DataSourceColumnMetaRepository,
                data_source_repo::DataSourceRepository,
                data_source_table_meta_repo::DataSourceTableMetaRepository,
            },
        },
        transtask::{
            handler::trans_task_handler::TransTaskPageQuery,
            job::job::TransTaskJob,
            model::trans_task::{
                TransTask, TransTaskBatchCreateBo, TransTaskCreateBo,
                TransTaskDetailVo, TransTaskListVo, TransTaskUpdateBo,
            },
            repository::trans_task_repo::TransTaskRepository,
        },
    },
    common::vo::page::{PageResult, normalize_paging, sanitize_sort},
    error::error::{AppError, AppResult},
    middleware::extractors::CurrentUser,
    util::{
        db_util::test_data_source_sync,
        duckdb_util::{ProgressTx, SyncProgressEvent, trans_table},
        job_util::{add_job_to_scheduler, remove_job_from_scheduler},
    },
};

#[derive(Clone)]
pub struct TransTaskService {
    trans_task_repo: TransTaskRepository,
    data_source_repo: DataSourceRepository,
    table_meta_repo: DataSourceTableMetaRepository,
    column_meta_repo: DataSourceColumnMetaRepository,
    data_dir: std::path::PathBuf,
    job_scheduler: Option<Arc<JobScheduler>>,
    encryption_key: String,
    task_events_tx:
        tokio::sync::broadcast::Sender<crate::app::TransTaskSyncedEvent>,
}

impl TransTaskService {
    pub fn new(infra: &Infrastructure) -> Self {
        Self {
            trans_task_repo: TransTaskRepository::new(infra.db.clone()),
            data_source_repo: DataSourceRepository::new(infra.db.clone()),
            table_meta_repo: DataSourceTableMetaRepository::new(
                infra.db.clone(),
            ),
            column_meta_repo: DataSourceColumnMetaRepository::new(
                infra.db.clone(),
            ),
            data_dir: infra.duckdb_data_dir.clone(),
            job_scheduler: infra.sched.clone(),
            encryption_key: infra.encryption_key.clone(),
            task_events_tx: infra.task_events_tx.clone(),
        }
    }

    /// 从元数据库中查找指定表的主键列（按列名有序拼接），用于优先级链第 2 步。
    async fn resolve_pk_from_metadata(
        &self,
        data_source_id: i64,
        table_name: &str,
    ) -> Option<String> {
        let table_meta = self
            .table_meta_repo
            .select_by_data_source_and_table(&data_source_id, table_name)
            .await
            .ok()??;

        let table_meta_id = table_meta.table_meta_id?;
        let pk_cols = self
            .column_meta_repo
            .select_pk_by_table_meta_id(&table_meta_id)
            .await
            .ok()?;

        if pk_cols.is_empty() {
            return None;
        }
        Some(
            pk_cols
                .iter()
                .map(|c| c.column_name.as_str())
                .collect::<Vec<_>>()
                .join(","),
        )
    }

    /// 解析同步主键，按优先级：用户配置 > 元数据 > None（由 trans_table 自动探测）。
    async fn resolve_effective_pk(
        &self,
        trans_task: &TransTaskDetailVo,
    ) -> Option<String> {
        // 1. 用户在任务配置中显式指定
        if let Some(pk) = trans_task
            .sync_primary_key
            .as_ref()
            .filter(|pk| !pk.trim().is_empty())
        {
            return Some(pk.clone());
        }
        // 2. 从已缓存的列元数据中读取（is_primary_key = 1）
        self.resolve_pk_from_metadata(
            trans_task.data_source_id,
            &trans_task.table_name,
        )
        .await
        // 3. None → trans_table 内部自动查源库（spawn_blocking 里执行）
    }

    #[tracing::instrument(skip(self))]
    pub async fn list(&self) -> AppResult<Vec<TransTaskListVo>> {
        let result = self.trans_task_repo.select_all().await?;
        tracing::info!(count = result.len(), "查询传输任务列表成功");
        Ok(result
            .into_iter()
            .map(|entity| entity.to_list_vo())
            .collect())
    }

    #[tracing::instrument(skip(self, query))]
    pub async fn page(
        &self,
        query: TransTaskPageQuery,
    ) -> AppResult<PageResult<TransTaskListVo>> {
        let (current, size, _offset) = normalize_paging(&query.page);
        tracing::info!(
            current = current,
            size = size,
            keyword = ?query.keyword,
            "传输任务分页查询"
        );
        let allowed = [
            ("transTaskId", "trans_task_id"),
            ("dataSourceId", "data_source_id"),
            ("tableName", "table_name"),
            ("taskStatus", "task_status"),
            ("createdDate", "created_date"),
        ];
        let mut order_by =
            sanitize_sort(&query.page.sort, &query.page.order, &allowed);
        if order_by.is_none() {
            order_by = Some("trans_task_id desc".to_string());
        }

        let order_str = order_by.as_deref().unwrap_or("trans_task_id desc");
        let page = self
            .trans_task_repo
            .select_page(&query, current as u64, size as u64, order_str)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "传输任务分页查询失败");
                e
            })?;
        let total = page.total as i64;
        if total <= 0 {
            return Ok(PageResult::empty(current, size));
        }
        let pages = (total + size - 1) / size;
        tracing::info!(
            total = total,
            pages = pages,
            current = current,
            size = size,
            "传输任务分页查询完成"
        );
        Ok(PageResult {
            records: page.records.into_iter().map(|e| e.to_list_vo()).collect(),
            total,
            pages,
            current,
            size,
        })
    }

    #[tracing::instrument(skip(self), fields(id = id))]
    pub async fn get_trans_task_by_id(
        &self,
        id: i64,
    ) -> AppResult<TransTaskDetailVo> {
        let entity =
            self.trans_task_repo
                .select_by_id(&id)
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!("传输任务不存在 (id={})", id))
                })?;
        tracing::info!(trans_task_id = id, "查询传输任务详情成功");
        Ok(entity.to_detail_vo())
    }

    #[tracing::instrument(skip(self, bo, current_user), fields(username = %current_user.username))]
    pub async fn insert_by_bo(
        &self,
        bo: TransTaskCreateBo,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let entity = TransTask::from_create_bo(bo, current_user);
        self.trans_task_repo.insert(&entity).await?;
        tracing::info!(
            trans_task_id = ?entity.trans_task_id,
            "创建传输任务成功"
        );
        Ok(())
    }

    #[tracing::instrument(skip(self, bo, current_user), fields(username = %current_user.username))]
    pub async fn insert_batch_by_bo(
        &self,
        bo: TransTaskBatchCreateBo,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let data_source_id = bo.data_source_id.ok_or_else(|| {
            AppError::BadRequest("数据源为必填项".to_string())
        })?;
        tracing::info!(
            data_source_id = data_source_id,
            table_count = bo.tables.len(),
            "开始批量创建传输任务"
        );
        let remark = bo.remark.clone();
        let task_cron = bo.task_cron.clone();
        let task_status = bo.task_status.clone();
        let sync_mode = bo.sync_mode.clone();
        let sync_strategy = bo.sync_strategy.clone();
        let sync_column = bo.sync_column.clone();
        let sync_primary_key = bo.sync_primary_key.clone();
        let entities: Vec<TransTask> = bo
            .tables
            .into_iter()
            .map(|t| {
                let single = TransTaskCreateBo {
                    data_source_id: Some(data_source_id),
                    table_name: t.table_name,
                    table_comment: t.table_comment,
                    remark: remark.clone(),
                    task_cron: task_cron.clone(),
                    task_status: task_status.clone(),
                    sync_mode: sync_mode.clone(),
                    sync_strategy: sync_strategy.clone(),
                    sync_column: sync_column.clone(),
                    sync_primary_key: sync_primary_key.clone(),
                };
                TransTask::from_create_bo(single, current_user)
            })
            .collect();

        self.trans_task_repo
            .insert_batch_in_transaction(&entities)
            .await?;
        tracing::info!(count = entities.len(), "批量创建传输任务成功");
        Ok(())
    }

    #[tracing::instrument(skip(self, bo, current_user), fields(username = %current_user.username))]
    pub async fn update_by_bo(
        &self,
        bo: TransTaskUpdateBo,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let trans_task_id = bo.trans_task_id.ok_or_else(|| {
            AppError::BadRequest("更新传输任务失败: 任务ID为空".to_string())
        })?;
        let mut entity = self
            .trans_task_repo
            .select_by_id(&trans_task_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "更新传输任务失败: 未找到任务 (id={})",
                    trans_task_id
                ))
            })?;

        entity.from_update_bo(bo, current_user);
        self.trans_task_repo
            .update_by_id(&entity, &trans_task_id)
            .await?;
        tracing::info!(trans_task_id = trans_task_id, "更新传输任务成功");
        Ok(())
    }

    #[tracing::instrument(skip(self, current_user), fields(id = id, username = %current_user.username))]
    pub async fn delete_by_id(
        &self,
        id: i64,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let mut entity = self
            .trans_task_repo
            .select_by_id(&id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "软删除失败: 未找到任务 (id={})",
                    id
                ))
            })?;

        entity.base_entity.delete(current_user.get_user_id());

        self.trans_task_repo.update_by_id(&entity, &id).await?;
        tracing::info!(trans_task_id = id, "软删除传输任务成功");
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(id = id))]
    pub async fn true_delete_by_id(&self, id: i64) -> AppResult<()> {
        self.trans_task_repo.delete_by_id(&id).await?;
        tracing::info!(trans_task_id = id, "彻底删除传输任务成功");
        Ok(())
    }

    // Helper method to fetch data source for a task
    #[tracing::instrument(skip(self))]
    async fn get_data_source_for_task(
        &self,
        data_source_id: i64,
    ) -> Option<DataSource> {
        match self.data_source_repo.select_by_id(&data_source_id).await {
            Ok(Some(data_source)) => {
                match data_source.to_decrypted(self.encryption_key.as_bytes()) {
                    Ok(vo) => Some(vo),
                    Err(e) => {
                        tracing::error!(
                            data_source_id = data_source_id,
                            error = %e,
                            "解密数据源失败"
                        );
                        None
                    }
                }
            }
            Ok(None) => {
                tracing::error!(
                    data_source_id = data_source_id,
                    "未找到数据源"
                );
                None
            }
            Err(e) => {
                tracing::error!(
                    data_source_id = data_source_id,
                    error = %e,
                    "获取数据源失败"
                );
                None
            }
        }
    }

    #[tracing::instrument(skip(self), fields(id = id))]
    pub async fn run_trans_task(&self, id: i64) -> AppResult<String> {
        tracing::info!(trans_task_id = id, "开始执行传输任务");
        // Get trans task
        let trans_task = self.get_trans_task_by_id(id).await?;

        // Get data source
        let data_source = self
            .get_data_source_for_task(trans_task.data_source_id)
            .await
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "未找到数据源: {}",
                    trans_task.data_source_id
                ))
            })?;

        // 优先级链（异步部分：用户配置 > 元数据）
        // 第 3 优先级（自动查源库）在 spawn_blocking 内的 trans_table 里执行
        let effective_pk = self.resolve_effective_pk(&trans_task).await;
        let trans_task = TransTaskDetailVo {
            sync_primary_key: effective_pk,
            ..trans_task
        };

        // 抢占同步锁(原子 CAS),拒绝并发触发
        let trans_task_id = trans_task.trans_task_id;
        let data_source_id = trans_task.data_source_id;
        let acquired = self
            .trans_task_repo
            .try_acquire_syncing(trans_task_id)
            .await?;
        if !acquired {
            return Err(AppError::Conflict(
                "任务正在同步中,请等待当前同步完成".to_string(),
            ));
        }

        // 同步的数据源连通性测试 + DuckDB 导出都是阻塞调用,
        // 放到 spawn_blocking 里执行,避免阻塞 tokio worker 线程,
        // 否则执行期间 /api/health 等接口会卡住报错。
        let data_dir = self.data_dir.clone();
        let join_result = tokio::task::spawn_blocking(move || {
            let (is_success, msg) = test_data_source_sync(&data_source);
            if !is_success {
                return Err(msg);
            }

            trans_table(&data_source, &trans_task, &data_dir, None)
                // {:#} 展开 anyhow 完整错误链，UI 任务信息里能看到根因
                .map_err(|e| format!("传输任务执行失败: {:#}", e))
        })
        .await;

        // 无论成败必须 release
        if let Err(e) =
            self.trans_task_repo.release_syncing(trans_task_id).await
        {
            tracing::error!(error = %e, "释放同步锁失败");
        }

        match join_result {
            Ok(Ok(sync_result)) => {
                let row_count = sync_result.row_count as i64;
                tracing::info!(
                    trans_task_id = trans_task_id,
                    row_count = row_count,
                    new_watermark = ?sync_result.new_watermark,
                    "传输任务执行成功"
                );
                // 根据是否有新水位值选择不同的持久化方式
                let persist_result =
                    if let Some(ref watermark) = sync_result.new_watermark {
                        self.trans_task_repo
                            .update_sync_state_by_id(
                                trans_task_id,
                                row_count,
                                watermark,
                            )
                            .await
                    } else {
                        self.trans_task_repo
                            .update_row_count_by_id(trans_task_id, row_count)
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
                        data_source_id,
                        row_count,
                    },
                );
                Ok("传输任务执行成功".to_string())
            }
            Ok(Err(msg)) => {
                tracing::error!(
                    trans_task_id = trans_task_id,
                    error = %msg,
                    "传输任务执行失败"
                );
                Err(AppError::Internal(msg))
            }
            Err(e) => {
                tracing::error!(
                    trans_task_id = trans_task_id,
                    error = %e,
                    "传输任务阻塞线程异常"
                );
                Err(AppError::Internal("传输任务执行异常".to_string()))
            }
        }
    }

    #[tracing::instrument(skip(self), fields(id = id))]
    pub async fn start_trans_task(&self, id: i64) -> AppResult<()> {
        tracing::info!(trans_task_id = id, "开始启动传输任务");
        // Get trans task
        let trans_task = self.get_trans_task_by_id(id).await?;

        // Get data source
        let data_source = self
            .get_data_source_for_task(trans_task.data_source_id)
            .await
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "未找到任务的数据源: {}",
                    trans_task.data_source_id
                ))
            })?;

        // Schedule the task
        let uuid = self.schedule_task(&data_source, &trans_task).await?;

        tracing::info!(
            trans_task_id = trans_task.trans_task_id,
            cron_expression = %trans_task.task_cron,
            job_uuid = %uuid,
            "已调度定时任务"
        );

        // Update task UUID
        if let Err(e) = self
            .trans_task_repo
            .start_trans_task_by_id(
                trans_task.trans_task_id,
                uuid.to_string().as_str(),
            )
            .await
        {
            tracing::error!(
                trans_task_id = trans_task.trans_task_id,
                error = %e,
                "更新任务UUID失败"
            );
        }

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(id = id))]
    pub async fn stop_trans_task(&self, id: i64) -> AppResult<()> {
        tracing::info!(trans_task_id = id, "开始停止传输任务");
        // Get trans task
        let trans_task = self
            .trans_task_repo
            .select_by_id(&id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("传输任务不存在 (id={})", id))
            })?;

        // Get trans task ID
        let trans_task_id = trans_task
            .trans_task_id
            .ok_or_else(|| AppError::Internal("无效的ID".to_string()))?;

        // Update task status in database
        self.trans_task_repo
            .stop_trans_task_by_id(trans_task_id)
            .await?;

        // Get task UUID
        let task_uuid = trans_task
            .task_uuid
            .ok_or_else(|| AppError::Internal("无效的任务".to_string()))?;

        // Parse UUID
        let uuid = Uuid::parse_str(&task_uuid).map_err(|e| {
            tracing::error!(
                task_uuid = %task_uuid,
                error = %e,
                "解析任务UUID失败"
            );
            AppError::Internal(format!("解析任务UUID失败: {}", e))
        })?;

        // Remove from scheduler
        let sched = self.job_scheduler.as_ref().ok_or_else(|| {
            AppError::Internal("任务调度器未初始化".to_string())
        })?;

        remove_job_from_scheduler(sched, &uuid).await.map_err(|e| {
            AppError::Internal(format!("停止任务失败: {:?}", e))
        })?;
        tracing::info!(task_uuid = %uuid, "成功停止任务");
        Ok(())
    }

    #[tracing::instrument(skip(self, data_source, trans_task))]
    pub async fn schedule_task(
        &self,
        data_source: &DataSource,
        trans_task: &TransTaskDetailVo,
    ) -> AppResult<Uuid> {
        match self.job_scheduler.as_ref() {
            Some(sched) => {
                let trans_task = trans_task.clone();
                let data_dir = self.data_dir.clone();

                let uuid = add_job_to_scheduler(
                    sched.as_ref(),
                    trans_task.task_cron.as_str(),
                    Arc::new(TransTaskJob {
                        data_source: data_source.clone(),
                        trans_task: trans_task.clone(),
                        data_dir,
                        trans_task_repo: self.trans_task_repo.clone(),
                        task_events_tx: self.task_events_tx.clone(),
                    }),
                )
                .await
                .map_err(|e| {
                    tracing::error!(
                        error = %e,
                        "添加任务到调度器失败"
                    );
                    AppError::Internal(format!("添加任务到调度器失败: {:?}", e))
                })?;

                tracing::info!(
                    job_uuid = %uuid,
                    cron_expression = %trans_task.task_cron,
                    "任务已添加到调度器"
                );
                Ok(uuid)
            }
            None => Err(AppError::Internal("任务调度器未初始化".to_string())),
        }
    }

    /// 启动时清理残留的 is_syncing=1(进程崩溃时未正常释放的)。
    /// 在 init_app 里 schedule_all_tasks 之前调用。
    #[tracing::instrument(skip(self))]
    pub async fn reset_all_syncing_on_startup(&self) -> AppResult<()> {
        let cleared = self.trans_task_repo.reset_all_syncing().await?;
        if cleared > 0 {
            tracing::warn!(
                cleared = cleared,
                "启动时清理了 {} 个残留的 is_syncing=1 标志(可能是上次进程异常退出)",
                cleared
            );
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn schedule_all_tasks(&self) -> AppResult<()> {
        tracing::info!("开始调度所有传输任务");
        let tasks = self
            .trans_task_repo
            .select_by_task_status("start")
            .await?
            .into_iter()
            .map(|entity| entity.to_list_vo())
            .collect::<Vec<_>>();
        let mut scheduled_count = 0;
        let mut failed_count = 0;

        for task in tasks {
            match self.start_trans_task(task.trans_task_id).await {
                Ok(_) => scheduled_count += 1,
                Err(e) => {
                    failed_count += 1;
                    tracing::error!(
                        trans_task_id = task.trans_task_id,
                        error = %e,
                        "调度任务失败"
                    );
                }
            }
        }

        tracing::info!(
            scheduled_count = scheduled_count,
            failed_count = failed_count,
            "任务调度完成"
        );

        if failed_count > 0 {
            Err(AppError::Internal(format!(
                "调度 {} 个任务失败",
                failed_count
            )))
        } else {
            Ok(())
        }
    }

    /// 带进度推送的同步任务执行（供 SSE 端点调用）。
    /// 通过 `progress_tx` 发送进度事件，完成/失败后发送 Done/Error 事件。
    #[tracing::instrument(skip(self, progress_tx))]
    pub async fn run_trans_task_with_progress(
        &self,
        id: i64,
        progress_tx: ProgressTx,
    ) -> AppResult<()> {
        // ── 获取任务 ──────────────────────────────────────────────────────────
        let trans_task = match self.get_trans_task_by_id(id).await {
            Ok(t) => t,
            Err(_) => {
                let _ = progress_tx
                    .send(SyncProgressEvent::Error {
                        message: "未找到传输任务".to_string(),
                    })
                    .await;
                return Ok(());
            }
        };

        // ── 获取数据源 ────────────────────────────────────────────────────────
        let data_source = match self
            .get_data_source_for_task(trans_task.data_source_id)
            .await
        {
            Some(ds) => ds,
            None => {
                let _ = progress_tx
                    .send(SyncProgressEvent::Error {
                        message: "未找到数据源".to_string(),
                    })
                    .await;
                return Ok(());
            }
        };

        // 优先级链（异步部分：用户配置 > 元数据）
        let effective_pk = self.resolve_effective_pk(&trans_task).await;
        let trans_task = TransTaskDetailVo {
            sync_primary_key: effective_pk,
            ..trans_task
        };

        // 抢占同步锁(原子 CAS),拒绝并发触发
        let trans_task_id = trans_task.trans_task_id;
        let data_source_id = trans_task.data_source_id;
        match self
            .trans_task_repo
            .try_acquire_syncing(trans_task_id)
            .await
        {
            Ok(true) => {}
            Ok(false) => {
                let _ = progress_tx
                    .send(SyncProgressEvent::Error {
                        message: "任务正在同步中,请等待当前同步完成"
                            .to_string(),
                    })
                    .await;
                return Ok(());
            }
            Err(e) => {
                let _ = progress_tx
                    .send(SyncProgressEvent::Error {
                        message: format!("获取同步锁失败: {}", e),
                    })
                    .await;
                return Ok(());
            }
        }

        // ── 在阻塞线程中执行同步 ──────────────────────────────────────────────
        let data_dir = self.data_dir.clone();
        let tx_clone = progress_tx.clone();

        let join_result = tokio::task::spawn_blocking(move || {
            let (is_success, msg) = test_data_source_sync(&data_source);
            if !is_success {
                let _ = tx_clone.blocking_send(SyncProgressEvent::Error {
                    message: msg.clone(),
                });
                return Err(msg);
            }
            trans_table(&data_source, &trans_task, &data_dir, Some(tx_clone))
                // {:#} 展开 anyhow 完整错误链，SSE Error 事件里能看到根因
                .map_err(|e| format!("传输任务执行失败: {:#}", e))
        })
        .await;

        // 无论成败必须 release
        if let Err(e) =
            self.trans_task_repo.release_syncing(trans_task_id).await
        {
            tracing::error!(error = %e, "释放同步锁失败");
        }

        match join_result {
            Ok(Ok(sync_result)) => {
                let row_count = sync_result.row_count as i64;
                let persist_result =
                    if let Some(ref watermark) = sync_result.new_watermark {
                        self.trans_task_repo
                            .update_sync_state_by_id(
                                trans_task_id,
                                row_count,
                                watermark,
                            )
                            .await
                    } else {
                        self.trans_task_repo
                            .update_row_count_by_id(trans_task_id, row_count)
                            .await
                    };
                if let Err(e) = persist_result {
                    tracing::error!(error = %e, "更新任务同步状态失败");
                }
                let _ = self.task_events_tx.send(
                    crate::app::TransTaskSyncedEvent {
                        task_id: trans_task_id,
                        data_source_id,
                        row_count,
                    },
                );
                let _ = progress_tx
                    .send(SyncProgressEvent::Done {
                        row_count,
                        new_watermark: sync_result.new_watermark,
                    })
                    .await;
            }
            Ok(Err(msg)) => {
                let _ = progress_tx
                    .send(SyncProgressEvent::Error { message: msg })
                    .await;
            }
            Err(e) => {
                let _ = progress_tx
                    .send(SyncProgressEvent::Error {
                        message: format!("执行线程异常: {}", e),
                    })
                    .await;
            }
        }
        Ok(())
    }
}
