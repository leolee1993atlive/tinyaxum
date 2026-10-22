use std::sync::Arc;

use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use uuid::Uuid;

use crate::common::job::job::CronJob;

/// 添加 Job 到调度器
pub async fn add_job_to_scheduler(
    sched: &JobScheduler,
    cron: &str,
    job: Arc<dyn CronJob>,
) -> Result<Uuid, JobSchedulerError> {
    let job_clone = Arc::clone(&job);

    sched
        .add(Job::new_async(cron, move |uuid, _l| {
            let job = Arc::clone(&job_clone);
            Box::pin(async move {
                job.execute(uuid).await;
            })
        })?)
        .await
}

/// 移除 Job 从调度器
pub async fn remove_job_from_scheduler(
    sched: &JobScheduler,
    uuid: &Uuid,
) -> Result<(), JobSchedulerError> {
    sched.remove(uuid).await
}
