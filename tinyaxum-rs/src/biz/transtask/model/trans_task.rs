use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    common::model::entity::{BaseEntity, BaseVo},
    middleware::extractors::CurrentUser,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransTask {
    pub trans_task_id: Option<i64>,
    pub data_source_id: i64,
    pub table_name: String,
    pub table_comment: String,
    pub remark: Option<String>,
    pub row_count: i64,
    pub last_trans_time: DateTime<Utc>,
    pub task_cron: String,
    pub task_status: String,
    pub task_uuid: Option<String>,
    pub sync_mode: String,
    pub sync_strategy: Option<String>,
    pub sync_column: Option<String>,
    pub sync_watermark: Option<String>,
    pub sync_primary_key: Option<String>,
    /// 是否正在执行同步 (0=空闲, 1=运行中); 由后端 acquire/release 维护,前端只读
    pub is_syncing: i32,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}

impl TransTask {
    pub fn from_create_bo(bo: TransTaskCreateBo, user: &CurrentUser) -> Self {
        TransTask {
            trans_task_id: None,
            data_source_id: bo.data_source_id.unwrap_or_default(),
            table_name: bo.table_name,
            table_comment: bo.table_comment,
            remark: bo.remark,
            row_count: 0,
            last_trans_time: Utc::now(),
            task_cron: bo.task_cron,
            task_status: bo.task_status,
            task_uuid: None,
            sync_mode: bo.sync_mode.unwrap_or_else(|| "full".to_string()),
            sync_strategy: bo.sync_strategy,
            sync_column: bo.sync_column,
            sync_watermark: None,
            sync_primary_key: bo.sync_primary_key,
            is_syncing: 0,
            base_entity: BaseEntity::new(user.get_user_id()),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_update_bo(
        &mut self,
        bo: TransTaskUpdateBo,
        user: &CurrentUser,
    ) {
        self.trans_task_id = bo.trans_task_id;
        self.data_source_id = bo.data_source_id.unwrap_or_default();
        self.table_name = bo.table_name;
        self.table_comment = bo.table_comment;
        self.remark = bo.remark;
        self.task_cron = bo.task_cron;
        self.task_status = bo.task_status;
        self.sync_mode = bo.sync_mode.unwrap_or_else(|| "full".to_string());
        self.sync_strategy = bo.sync_strategy;
        self.sync_column = bo.sync_column;
        self.sync_primary_key = bo.sync_primary_key;
        self.base_entity.update(user.get_user_id());
    }

    pub fn to_list_vo(&self) -> TransTaskListVo {
        TransTaskListVo {
            trans_task_id: self.trans_task_id.unwrap_or_default(),
            data_source_id: self.data_source_id,
            table_name: self.table_name.clone(),
            table_comment: self.table_comment.clone(),
            remark: self.remark.clone(),
            row_count: self.row_count,
            last_trans_time: self.last_trans_time,
            task_cron: self.task_cron.clone(),
            task_status: self.task_status.clone(),
            sync_mode: self.sync_mode.clone(),
            sync_strategy: self.sync_strategy.clone(),
            sync_column: self.sync_column.clone(),
            sync_watermark: self.sync_watermark.clone(),
            sync_primary_key: self.sync_primary_key.clone(),
            is_syncing: self.is_syncing,
            base_entity: BaseVo::new(self.base_entity.clone()),
        }
    }

    pub fn to_detail_vo(&self) -> TransTaskDetailVo {
        TransTaskDetailVo {
            trans_task_id: self.trans_task_id.unwrap_or_default(),
            data_source_id: self.data_source_id,
            table_name: self.table_name.clone(),
            table_comment: self.table_comment.clone(),
            remark: self.remark.clone(),
            row_count: self.row_count,
            last_trans_time: self.last_trans_time,
            task_cron: self.task_cron.clone(),
            task_status: self.task_status.clone(),
            sync_mode: self.sync_mode.clone(),
            sync_strategy: self.sync_strategy.clone(),
            sync_column: self.sync_column.clone(),
            sync_watermark: self.sync_watermark.clone(),
            sync_primary_key: self.sync_primary_key.clone(),
            is_syncing: self.is_syncing,
            base_entity: BaseVo::new(self.base_entity.clone()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TransTaskCreateBo {
    #[validate(required)]
    pub data_source_id: Option<i64>,
    #[validate(length(min = 1, message = "table_name cannot be empty"))]
    pub table_name: String,
    #[validate(length(min = 1, message = "table_comment cannot be empty"))]
    pub table_comment: String,
    pub remark: Option<String>,
    #[validate(length(min = 1, message = "task_cron cannot be empty"))]
    pub task_cron: String,
    pub task_status: String,
    pub sync_mode: Option<String>,
    pub sync_strategy: Option<String>,
    pub sync_column: Option<String>,
    pub sync_primary_key: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TransTaskUpdateBo {
    #[validate(required)]
    pub trans_task_id: Option<i64>,
    #[validate(required)]
    pub data_source_id: Option<i64>,
    #[validate(length(min = 1, message = "table_name cannot be empty"))]
    pub table_name: String,
    #[validate(length(min = 1, message = "table_comment cannot be empty"))]
    pub table_comment: String,
    pub remark: Option<String>,
    #[validate(length(min = 1, message = "task_cron cannot be empty"))]
    pub task_cron: String,
    pub task_status: String,
    pub sync_mode: Option<String>,
    pub sync_strategy: Option<String>,
    pub sync_column: Option<String>,
    pub sync_primary_key: Option<String>,
}

/// 单条批量创建中的表项（与 [TransTaskCreateBo] 中表字段一致）
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TransTaskBatchTableBo {
    #[validate(length(min = 1, message = "table_name cannot be empty"))]
    pub table_name: String,
    #[validate(length(min = 1, message = "table_comment cannot be empty"))]
    pub table_comment: String,
}

/// 批量创建传输任务（共享数据源、Cron、备注与状态）
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TransTaskBatchCreateBo {
    #[validate(required)]
    pub data_source_id: Option<i64>,
    #[validate(length(
        min = 1,
        max = 500,
        message = "tables must contain 1 to 500 items"
    ))]
    #[validate(nested)]
    pub tables: Vec<TransTaskBatchTableBo>,
    #[validate(length(min = 1, message = "task_cron cannot be empty"))]
    pub task_cron: String,
    pub remark: Option<String>,
    #[validate(length(min = 1, message = "task_status cannot be empty"))]
    pub task_status: String,
    pub sync_mode: Option<String>,
    pub sync_strategy: Option<String>,
    pub sync_column: Option<String>,
    pub sync_primary_key: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TransTaskListVo {
    pub trans_task_id: i64,
    pub data_source_id: i64,
    pub table_name: String,
    pub table_comment: String,
    pub remark: Option<String>,
    pub row_count: i64,
    pub last_trans_time: DateTime<Utc>,
    pub task_cron: String,
    pub task_status: String,
    pub sync_mode: String,
    pub sync_strategy: Option<String>,
    pub sync_column: Option<String>,
    pub sync_watermark: Option<String>,
    pub sync_primary_key: Option<String>,
    pub is_syncing: i32,
    #[serde(flatten)]
    pub base_entity: BaseVo,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TransTaskDetailVo {
    pub trans_task_id: i64,
    pub data_source_id: i64,
    pub table_name: String,
    pub table_comment: String,
    pub remark: Option<String>,
    pub row_count: i64,
    pub last_trans_time: DateTime<Utc>,
    pub task_cron: String,
    pub task_status: String,
    pub sync_mode: String,
    pub sync_strategy: Option<String>,
    pub sync_column: Option<String>,
    pub sync_watermark: Option<String>,
    pub sync_primary_key: Option<String>,
    pub is_syncing: i32,
    #[serde(flatten)]
    pub base_entity: BaseVo,
}
