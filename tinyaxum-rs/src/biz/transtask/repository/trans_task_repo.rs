use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, Order,
    PaginatorTrait, QueryFilter, QueryOrder, TransactionTrait,
    prelude::Expr,
};

use crate::{
    biz::transtask::{
        handler::trans_task_handler::TransTaskPageQuery,
        model::{
            trans_task::TransTask,
            trans_task_entity::{
                ActiveModel, Column as TransTaskColumn, Entity as BizTransTask,
                Model,
            },
        },
    },
    common::{model::entity::BaseEntity, repo::str_trim_or_empty},
};

impl From<&Model> for TransTask {
    fn from(m: &Model) -> Self {
        TransTask {
            trans_task_id: Some(m.trans_task_id),
            data_source_id: m.data_source_id,
            table_name: m.table_name.clone(),
            table_comment: m.table_comment.clone(),
            remark: m.remark.clone(),
            row_count: m.row_count,
            last_trans_time: m.last_trans_time,
            task_cron: m.task_cron.clone(),
            task_status: m.task_status.clone(),
            task_uuid: m.task_uuid.clone(),
            sync_mode: m.sync_mode.clone(),
            sync_strategy: m.sync_strategy.clone(),
            sync_column: m.sync_column.clone(),
            sync_watermark: m.sync_watermark.clone(),
            sync_primary_key: m.sync_primary_key.clone(),
            is_syncing: m.is_syncing,
            base_entity: BaseEntity {
                version: m.version,
                created_by: m.created_by,
                created_date: m.created_date,
                last_modified_by: m.last_modified_by,
                last_modified_date: m.last_modified_date,
                deleted_by: m.deleted_by,
                deleted_date: m.deleted_date,
            },
        }
    }
}

fn to_active(task: &TransTask) -> ActiveModel {
    ActiveModel {
        trans_task_id: match task.trans_task_id {
            Some(id) => Set(id),
            None => NotSet,
        },
        data_source_id: Set(task.data_source_id),
        table_name: Set(task.table_name.clone()),
        table_comment: Set(task.table_comment.clone()),
        remark: Set(task.remark.clone()),
        row_count: Set(task.row_count),
        last_trans_time: Set(task.last_trans_time),
        task_cron: Set(task.task_cron.clone()),
        task_status: Set(task.task_status.clone()),
        task_uuid: Set(task.task_uuid.clone()),
        sync_mode: Set(task.sync_mode.clone()),
        sync_strategy: Set(task.sync_strategy.clone()),
        sync_column: Set(task.sync_column.clone()),
        sync_watermark: Set(task.sync_watermark.clone()),
        sync_primary_key: Set(task.sync_primary_key.clone()),
        is_syncing: Set(task.is_syncing),
        version: Set(task.base_entity.version),
        created_by: Set(task.base_entity.created_by),
        created_date: Set(task.base_entity.created_date),
        last_modified_by: Set(task.base_entity.last_modified_by),
        last_modified_date: Set(task.base_entity.last_modified_date),
        deleted_by: Set(task.base_entity.deleted_by),
        deleted_date: Set(task.base_entity.deleted_date),
    }
}

fn active_filter() -> Condition {
    Condition::all()
        .add(TransTaskColumn::DeletedBy.is_null())
        .add(TransTaskColumn::DeletedDate.is_null())
}

/// Falls back to `trans_task_id desc`.
fn parse_order(order_by: &str) -> (TransTaskColumn, Order) {
    let mut parts = order_by.split_whitespace();
    let col = match parts.next().unwrap_or("trans_task_id") {
        "trans_task_id" => TransTaskColumn::TransTaskId,
        "data_source_id" => TransTaskColumn::DataSourceId,
        "table_name" => TransTaskColumn::TableName,
        "task_status" => TransTaskColumn::TaskStatus,
        "created_date" => TransTaskColumn::CreatedDate,
        _ => TransTaskColumn::TransTaskId,
    };
    let desc = !parts.next().is_some_and(|d| d.eq_ignore_ascii_case("asc"));
    if desc {
        (col, Order::Desc)
    } else {
        (col, Order::Asc)
    }
}

pub struct TransTaskPage {
    pub records: Vec<TransTask>,
    pub total: u64,
}

#[derive(Clone)]
pub struct TransTaskRepository {
    db: DatabaseConnection,
}

impl TransTaskRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn select_all(&self) -> Result<Vec<TransTask>, sea_orm::DbErr> {
        let models = BizTransTask::find()
            .filter(active_filter())
            .order_by_asc(TransTaskColumn::TransTaskId)
            .all(&self.db)
            .await?;
        Ok(models.iter().map(TransTask::from).collect())
    }

    pub async fn select_page(
        &self,
        query: &TransTaskPageQuery,
        page_no: u64,
        page_size: u64,
        order_by: &str,
    ) -> Result<TransTaskPage, sea_orm::DbErr> {
        let keyword = str_trim_or_empty(&query.keyword);
        let table_name = str_trim_or_empty(&query.table_name);
        let table_comment = str_trim_or_empty(&query.table_comment);
        let task_status = str_trim_or_empty(&query.task_status);
        let data_source_id = query.data_source_id.unwrap_or(0);

        let mut select = BizTransTask::find().filter(active_filter());

        if data_source_id > 0 {
            select =
                select.filter(TransTaskColumn::DataSourceId.eq(data_source_id));
        }
        if !table_name.is_empty() {
            select =
                select.filter(TransTaskColumn::TableName.contains(&table_name));
        }
        if !table_comment.is_empty() {
            select = select
                .filter(TransTaskColumn::TableComment.contains(&table_comment));
        }
        if !task_status.is_empty() {
            select =
                select.filter(TransTaskColumn::TaskStatus.eq(&task_status));
        }
        if !keyword.is_empty() {
            select = select.filter(
                Condition::any()
                    .add(TransTaskColumn::TableName.contains(&keyword))
                    .add(TransTaskColumn::TableComment.contains(&keyword)),
            );
        }

        let (order_col, order_dir) = parse_order(order_by);
        select = select.order_by(order_col, order_dir);

        // Paginate: count + fetch
        let paginator = select.paginate(&self.db, page_size);
        let total = paginator.num_items().await?;
        let models = paginator.fetch_page(page_no.saturating_sub(1)).await?;

        Ok(TransTaskPage {
            records: models.iter().map(TransTask::from).collect(),
            total,
        })
    }

    /// Type-safe replacement for the original `select_by_map(Value)`.
    /// Filters by task_status (the only dynamic map usage was
    /// `value! {"task_status": "start"}` in `schedule_all_tasks`).
    pub async fn select_by_task_status(
        &self,
        task_status: &str,
    ) -> Result<Vec<TransTask>, sea_orm::DbErr> {
        let models = BizTransTask::find()
            .filter(active_filter())
            .filter(TransTaskColumn::TaskStatus.eq(task_status))
            .order_by_desc(TransTaskColumn::TransTaskId)
            .all(&self.db)
            .await?;
        Ok(models.iter().map(TransTask::from).collect())
    }

    pub async fn select_by_id(
        &self,
        trans_task_id: &i64,
    ) -> Result<Option<TransTask>, sea_orm::DbErr> {
        let model = BizTransTask::find_by_id(*trans_task_id)
            .filter(active_filter())
            .one(&self.db)
            .await?;
        Ok(model.as_ref().map(TransTask::from))
    }

    pub async fn insert(
        &self,
        data_source: &TransTask,
    ) -> Result<(), sea_orm::DbErr> {
        let active = to_active(data_source);
        BizTransTask::insert(active).exec(&self.db).await?;
        Ok(())
    }

    /// Batch insert inside a single database transaction.
    /// All-or-nothing semantics are preserved.
    pub async fn insert_batch_in_transaction(
        &self,
        entities: &[TransTask],
    ) -> Result<(), sea_orm::DbErr> {
        if entities.is_empty() {
            return Ok(());
        }

        let active_models: Vec<ActiveModel> =
            entities.iter().map(to_active).collect();

        self.db
            .transaction(|txn| {
                Box::pin(async move {
                    BizTransTask::insert_many(active_models).exec(txn).await?;
                    Ok::<_, sea_orm::DbErr>(())
                })
            })
            .await
            .map_err(|e| match e {
                sea_orm::TransactionError::Connection(err) => err,
                sea_orm::TransactionError::Transaction(err) => err,
            })
    }

    pub async fn update_by_id(
        &self,
        data_source: &TransTask,
        _trans_task_id: &i64,
    ) -> Result<(), sea_orm::DbErr> {
        let active = to_active(data_source);
        let _ = BizTransTask::update(active)
            .validate()?
            .filter(active_filter())
            .exec(&self.db)
            .await?;
        Ok(())
    }

    pub async fn delete_by_id(
        &self,
        trans_task_id: &i64,
    ) -> Result<(), sea_orm::DbErr> {
        BizTransTask::delete_by_id(*trans_task_id)
            .exec(&self.db)
            .await?;
        Ok(())
    }

    pub async fn start_trans_task_by_id(
        &self,
        trans_task_id: i64,
        task_uuid: &str,
    ) -> Result<(), sea_orm::DbErr> {
        BizTransTask::update_many()
            .col_expr(TransTaskColumn::TaskStatus, Expr::value("start"))
            .col_expr(TransTaskColumn::TaskUuid, Expr::value(task_uuid))
            .filter(TransTaskColumn::TransTaskId.eq(trans_task_id))
            .filter(active_filter())
            .exec(&self.db)
            .await?;
        Ok(())
    }

    pub async fn stop_trans_task_by_id(
        &self,
        trans_task_id: i64,
    ) -> Result<(), sea_orm::DbErr> {
        BizTransTask::update_many()
            .col_expr(TransTaskColumn::TaskStatus, Expr::value("stop"))
            .col_expr(
                TransTaskColumn::TaskUuid,
                Expr::value::<Option<String>>(None),
            )
            .filter(TransTaskColumn::TransTaskId.eq(trans_task_id))
            .filter(active_filter())
            .exec(&self.db)
            .await?;
        Ok(())
    }

    /// After a successful transfer, update the row count and refresh
    /// the last transfer time to now.
    pub async fn update_row_count_by_id(
        &self,
        trans_task_id: i64,
        row_count: i64,
    ) -> Result<(), sea_orm::DbErr> {
        BizTransTask::update_many()
            .col_expr(TransTaskColumn::RowCount, Expr::value(row_count))
            .col_expr(
                TransTaskColumn::LastTransTime,
                Expr::cust("CURRENT_TIMESTAMP"),
            )
            .filter(TransTaskColumn::TransTaskId.eq(trans_task_id))
            .filter(active_filter())
            .exec(&self.db)
            .await?;
        Ok(())
    }

    /// 原子获取同步执行锁: 仅当 is_syncing=0 时置 1,返回是否抢到锁。
    /// 用于阻止同一任务被 SSE/cron/手动并发触发。
    pub async fn try_acquire_syncing(
        &self,
        trans_task_id: i64,
    ) -> Result<bool, sea_orm::DbErr> {
        let result = BizTransTask::update_many()
            .col_expr(TransTaskColumn::IsSyncing, Expr::value(1))
            .filter(TransTaskColumn::TransTaskId.eq(trans_task_id))
            .filter(TransTaskColumn::IsSyncing.eq(0))
            .filter(active_filter())
            .exec(&self.db)
            .await?;
        Ok(result.rows_affected == 1)
    }

    /// 释放同步执行锁(置 is_syncing=0)。同步结束(成功或失败)都要调用。
    pub async fn release_syncing(
        &self,
        trans_task_id: i64,
    ) -> Result<(), sea_orm::DbErr> {
        BizTransTask::update_many()
            .col_expr(TransTaskColumn::IsSyncing, Expr::value(0))
            .filter(TransTaskColumn::TransTaskId.eq(trans_task_id))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    /// 启动时清理所有残留的 is_syncing=1 标志(进程崩溃时未正常 release 的)。
    pub async fn reset_all_syncing(&self) -> Result<u64, sea_orm::DbErr> {
        let result = BizTransTask::update_many()
            .col_expr(TransTaskColumn::IsSyncing, Expr::value(0))
            .filter(TransTaskColumn::IsSyncing.eq(1))
            .exec(&self.db)
            .await?;
        Ok(result.rows_affected)
    }

    /// After an incremental sync, update row count, watermark value and
    /// last transfer time.
    pub async fn update_sync_state_by_id(
        &self,
        trans_task_id: i64,
        row_count: i64,
        sync_watermark: &str,
    ) -> Result<(), sea_orm::DbErr> {
        BizTransTask::update_many()
            .col_expr(TransTaskColumn::RowCount, Expr::value(row_count))
            .col_expr(
                TransTaskColumn::SyncWatermark,
                Expr::value(sync_watermark),
            )
            .col_expr(
                TransTaskColumn::LastTransTime,
                Expr::cust("CURRENT_TIMESTAMP"),
            )
            .filter(TransTaskColumn::TransTaskId.eq(trans_task_id))
            .filter(active_filter())
            .exec(&self.db)
            .await?;
        Ok(())
    }
}
