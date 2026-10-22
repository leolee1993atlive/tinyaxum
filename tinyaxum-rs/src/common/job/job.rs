use std::pin::Pin;

use uuid::Uuid;

/// Job 接口定义,类似 Spring Quartz 的 Job 接口
pub trait CronJob: Send + Sync {
    fn execute(
        &self,
        uuid: Uuid,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}
