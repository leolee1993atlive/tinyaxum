use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use crate::{
    error::error::{AppError, AppResult},
    util::duckdb_util::validate_identifier,
};

/// 单个数据源下的"表名 → 文件路径"映射。
/// key 为 lowercase 表名（大小写不敏感匹配），value 为 (原始大小写的表名, 文件路径)。
pub type TableMap = HashMap<String, (String, PathBuf)>;

struct CacheEntry {
    refreshed_at: Instant,
    tables: Arc<TableMap>,
}

/// `<data_dir>/src_<code>/*.duckdb` 的目录扫描结果缓存。
///
/// - 按 `source_code` 维度缓存表清单，TTL 内复用
/// - 数据导入完成后可调用 `invalidate(source_code)` 主动失效
pub struct DuckdbCatalog {
    data_dir: PathBuf,
    ttl: Duration,
    cache: RwLock<HashMap<String, CacheEntry>>,
}

impl DuckdbCatalog {
    pub fn new(data_dir: PathBuf, ttl: Duration) -> Self {
        Self {
            data_dir,
            ttl,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// 获取某数据源的表映射（命中缓存直接返回，过期则重新扫描）。
    pub fn lookup(&self, source_code: &str) -> AppResult<Arc<TableMap>> {
        validate_identifier(source_code).map_err(|e| {
            AppError::Internal(format!(
                "数据源 code 含非法字符: {} - {}",
                source_code, e
            ))
        })?;

        if let Ok(read) = self.cache.read()
            && let Some(entry) = read.get(source_code)
            && entry.refreshed_at.elapsed() < self.ttl
        {
            return Ok(entry.tables.clone());
        }

        let tables = Arc::new(self.scan_dir(source_code)?);
        if let Ok(mut write) = self.cache.write() {
            write.insert(
                source_code.to_string(),
                CacheEntry {
                    refreshed_at: Instant::now(),
                    tables: tables.clone(),
                },
            );
        }
        Ok(tables)
    }

    fn scan_dir(&self, source_code: &str) -> AppResult<TableMap> {
        let src_dir = self.data_dir.join(format!("src_{}", source_code));
        if !src_dir.is_dir() {
            return Err(AppError::Internal(format!(
                "数据源 '{}' 的 DuckDB 数据目录不存在: {}",
                source_code,
                src_dir.display(),
            )));
        }

        let mut available: TableMap = HashMap::new();
        let entries = std::fs::read_dir(&src_dir).map_err(|e| {
            AppError::Internal(format!(
                "读取目录 '{}' 失败: {}",
                src_dir.display(),
                e
            ))
        })?;
        for entry in entries {
            let entry = entry.map_err(|e| {
                AppError::Internal(format!("枚举目录条目失败: {}", e))
            })?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("duckdb") {
                continue;
            }
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) if validate_identifier(s).is_ok() => s.to_string(),
                _ => continue,
            };
            available.insert(stem.to_lowercase(), (stem, path));
        }
        Ok(available)
    }
}
