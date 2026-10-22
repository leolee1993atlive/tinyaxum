use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
};

use chrono::NaiveDate;
use redis::AsyncCommands;

use crate::{
    app::Infrastructure,
    biz::sqlquery::{
        repository::sql_query_repo::SqlQueryRepository,
        service::sql_query_service::SqlQueryService,
    },
    common::vo::sql_run::SqlRunResultVo,
    error::error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct DashboardQueryService {
    redis_pool: deadpool_redis::Pool,
    sql_query_repo: SqlQueryRepository,
    sql_query_service: SqlQueryService,
    ttl_secs: u64,
    max_rows: i64,
}

impl DashboardQueryService {
    pub fn new(
        infra: &Infrastructure,
        sql_query_service: SqlQueryService,
    ) -> Self {
        Self {
            redis_pool: infra.pool.redis_pool.clone(),
            sql_query_repo: SqlQueryRepository::new(infra.db.clone()),
            sql_query_service,
            ttl_secs: infra.dashboard_cache_ttl_secs,
            max_rows: infra.dashboard_cache_max_rows,
        }
    }

    pub fn params_hash(params: &HashMap<String, String>) -> String {
        let mut sorted: Vec<(&String, &String)> = params.iter().collect();
        sorted.sort();
        let mut hasher = DefaultHasher::new();
        for (k, v) in sorted {
            k.hash(&mut hasher);
            v.hash(&mut hasher);
        }
        format!("{:016x}", hasher.finish())
    }

    pub fn cache_key(
        month: &str,
        dash_epoch: i64,
        sql_query_id: i64,
        src_ver: i64,
        params_hash: &str,
    ) -> String {
        format!(
            "dashq:{month}:{dash_epoch}:{sql_query_id}:{src_ver}:{params_hash}"
        )
    }

    fn dsver_key(data_source_id: i64) -> String {
        format!("dsver:{data_source_id}")
    }

    fn dash_epoch_key(month: &str) -> String {
        format!("dashEpoch:{month}")
    }

    pub fn month_date_range(month: &str) -> AppResult<(String, String)> {
        let parts: Vec<&str> = month.split('-').collect();
        if parts.len() != 2 {
            return Err(AppError::BadRequest(format!(
                "月份格式应为 YYYY-MM: {month}"
            )));
        }
        let y: i32 = parts[0]
            .parse()
            .map_err(|_| AppError::BadRequest(format!("非法年份: {month}")))?;
        let m: u32 = parts[1]
            .parse()
            .map_err(|_| AppError::BadRequest(format!("非法月份: {month}")))?;
        if !(1..=12).contains(&m) {
            return Err(AppError::BadRequest(format!("月份越界: {month}")));
        }
        let start = NaiveDate::from_ymd_opt(y, m, 1).ok_or_else(|| {
            AppError::BadRequest(format!("非法日期: {month}"))
        })?;
        let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
        let next_first =
            NaiveDate::from_ymd_opt(ny, nm, 1).ok_or_else(|| {
                AppError::BadRequest(format!("非法日期: {month}"))
            })?;
        let last = next_first.pred_opt().ok_or_else(|| {
            AppError::BadRequest(format!("非法日期: {month}"))
        })?;
        Ok((
            start.format("%Y-%m-%d").to_string(),
            last.format("%Y-%m-%d").to_string(),
        ))
    }

    async fn get_counter(&self, key: &str) -> i64 {
        let mut conn = match self.redis_pool.get().await {
            Ok(c) => c,
            Err(_) => return 0,
        };
        match conn.get::<_, Option<i64>>(key).await {
            Ok(Some(v)) => v,
            _ => 0,
        }
    }

    pub async fn bump_source_version(&self, data_source_id: i64) {
        let mut conn = match self.redis_pool.get().await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "获取 Redis 连接失败,跳过 dsver 自增"
                );
                return;
            }
        };
        if let Err(e) = conn
            .incr::<_, _, ()>(Self::dsver_key(data_source_id), 1)
            .await
        {
            tracing::warn!(error = %e, data_source_id, "dsver 自增失败");
        } else {
            tracing::info!(data_source_id, "首页缓存:数据源版本 +1");
        }
    }

    pub async fn invalidate_month(&self, month: &str) -> AppResult<()> {
        let mut conn = match self.redis_pool.get().await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "获取 Redis 连接失败,跳过失效"
                );
                return Ok(());
            }
        };
        if let Err(e) =
            conn.incr::<_, _, ()>(Self::dash_epoch_key(month), 1).await
        {
            tracing::warn!(error = %e, month, "dashEpoch 自增失败");
        }
        Ok(())
    }

    #[tracing::instrument(skip(self, params), fields(id = sql_query_id, month = month))]
    pub async fn run_cached(
        &self,
        sql_query_id: i64,
        month: &str,
        params: HashMap<String, String>,
        no_cache: bool,
    ) -> AppResult<(SqlRunResultVo, bool)> {
        let ds_id = self
            .sql_query_repo
            .select_by_id(&sql_query_id)
            .await?
            .and_then(|e| e.data_source_id)
            .unwrap_or(0);

        let phash = Self::params_hash(&params);
        let dash_epoch = self.get_counter(&Self::dash_epoch_key(month)).await;
        let src_ver = self.get_counter(&Self::dsver_key(ds_id)).await;
        let key =
            Self::cache_key(month, dash_epoch, sql_query_id, src_ver, &phash);

        if !no_cache
            && let Ok(mut conn) = self.redis_pool.get().await
            && let Ok(Some(json)) = conn.get::<_, Option<String>>(&key).await
            && let Ok(vo) = serde_json::from_str::<SqlRunResultVo>(&json)
        {
            tracing::info!(sql_query_id, "首页缓存命中");
            return Ok((vo, true));
        }

        let out = self
            .sql_query_service
            .run_for_dashboard(sql_query_id, Some(params), self.max_rows)
            .await?;

        if let Ok(mut conn) = self.redis_pool.get().await
            && let Ok(json) = serde_json::to_string(&out)
        {
            let _ = conn.set_ex::<_, _, ()>(&key, json, self.ttl_secs).await;
        }

        Ok((out, false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_hash_is_order_independent_and_stable() {
        let mut a = HashMap::new();
        a.insert("startDate".to_string(), "2026-05-01".to_string());
        a.insert("endDate".to_string(), "2026-05-31".to_string());
        let mut b = HashMap::new();
        b.insert("endDate".to_string(), "2026-05-31".to_string());
        b.insert("startDate".to_string(), "2026-05-01".to_string());
        assert_eq!(
            DashboardQueryService::params_hash(&a),
            DashboardQueryService::params_hash(&b)
        );
        let mut c = HashMap::new();
        c.insert("startDate".to_string(), "2026-06-01".to_string());
        c.insert("endDate".to_string(), "2026-06-30".to_string());
        assert_ne!(
            DashboardQueryService::params_hash(&a),
            DashboardQueryService::params_hash(&c)
        );
    }

    #[test]
    fn cache_key_format() {
        let k = DashboardQueryService::cache_key("2026-05", 3, 42, 7, "abc123");
        assert_eq!(k, "dashq:2026-05:3:42:7:abc123");
    }

    #[test]
    fn month_date_range_ok() {
        assert_eq!(
            DashboardQueryService::month_date_range("2026-05").unwrap(),
            ("2026-05-01".to_string(), "2026-05-31".to_string())
        );
        assert_eq!(
            DashboardQueryService::month_date_range("2024-02").unwrap(),
            ("2024-02-01".to_string(), "2024-02-29".to_string())
        );
        assert_eq!(
            DashboardQueryService::month_date_range("2026-12").unwrap(),
            ("2026-12-01".to_string(), "2026-12-31".to_string())
        );
    }

    #[test]
    fn month_date_range_rejects_bad_input() {
        assert!(DashboardQueryService::month_date_range("2026").is_err());
        assert!(DashboardQueryService::month_date_range("2026-13").is_err());
        assert!(DashboardQueryService::month_date_range("2026-00").is_err());
        assert!(DashboardQueryService::month_date_range("abc-de").is_err());
    }
}
