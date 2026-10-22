use std::{collections::HashMap, time::Duration};

use redis::AsyncCommands;

use crate::{
    app::Infrastructure,
    biz::datasource::{
        handler::data_source_handler::DataSourcePageQuery,
        model::{
            data_source::{
                DataSource, DataSourceCreateBo, DataSourceDetailVo,
                DataSourceListVo, DataSourceUpdateBo,
            },
            data_source_column_meta::DataSourceColumnMeta,
            data_source_schema_vo::{
                DataSourceColumnSchemaVo, DataSourceSchemaVo,
                DataSourceTableSchemaVo,
            },
            data_source_table_meta::DataSourceTableMeta,
        },
        repository::{
            data_source_column_meta_repo::DataSourceColumnMetaRepository,
            data_source_repo::DataSourceRepository,
            data_source_table_meta_repo::DataSourceTableMetaRepository,
        },
    },
    common::{
        model::entity::BaseEntity,
        vo::page::{PageResult, normalize_paging, sanitize_sort},
    },
    error::error::{AppError, AppResult},
    middleware::extractors::CurrentUser,
    util::{
        db_util::test_data_source,
        schema_introspection::{introspect_mysql, introspect_postgresql},
    },
};

#[derive(Clone)]
pub struct DataSourceService {
    data_source_repo: DataSourceRepository,
    table_meta_repo: DataSourceTableMetaRepository,
    column_meta_repo: DataSourceColumnMetaRepository,
    redis_pool: deadpool_redis::Pool,
    encryption_key: String,
}

impl DataSourceService {
    pub fn new(infra: &Infrastructure) -> Self {
        Self {
            data_source_repo: DataSourceRepository::new(infra.db.clone()),
            table_meta_repo: DataSourceTableMetaRepository::new(
                infra.db.clone(),
            ),
            column_meta_repo: DataSourceColumnMetaRepository::new(
                infra.db.clone(),
            ),
            redis_pool: infra.pool.redis_pool.clone(),
            encryption_key: infra.encryption_key.clone(),
        }
    }

    fn schema_cache_key(data_source_id: i64) -> String {
        format!("ds:schema:{}", data_source_id)
    }

    /// 读元数据缓存；任何 Redis/反序列化错误都降级为"未命中"，不影响主流程
    async fn read_schema_cache(&self, key: &str) -> Option<DataSourceSchemaVo> {
        let _timer =
            crate::util::metrics::RedisOpTimer::new("schema_cache_get");
        let mut conn = match self.redis_pool.get().await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(error = %e, "获取 Redis 连接失败，跳过读缓存");
                return None;
            }
        };
        let cached: Option<String> = match conn.get(key).await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, key, "读取 Redis 元数据缓存失败");
                return None;
            }
        };
        let json = cached?;
        match serde_json::from_str(&json) {
            Ok(vo) => Some(vo),
            Err(e) => {
                tracing::warn!(error = %e, key, "反序列化元数据缓存失败");
                None
            }
        }
    }

    /// 写元数据缓存（无过期）；失败仅记录，不影响主流程
    async fn write_schema_cache(&self, key: &str, vo: &DataSourceSchemaVo) {
        let _timer =
            crate::util::metrics::RedisOpTimer::new("schema_cache_set");
        let json = match serde_json::to_string(vo) {
            Ok(j) => j,
            Err(e) => {
                tracing::warn!(error = %e, key, "序列化元数据缓存失败");
                return;
            }
        };
        let mut conn = match self.redis_pool.get().await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(error = %e, "获取 Redis 连接失败，跳过写缓存");
                return;
            }
        };
        if let Err(e) = conn.set::<_, _, ()>(key, json).await {
            tracing::warn!(error = %e, key, "写入 Redis 元数据缓存失败");
        }
    }

    /// 删元数据缓存；失败仅记录
    async fn evict_schema_cache(&self, data_source_id: i64) {
        let _timer =
            crate::util::metrics::RedisOpTimer::new("schema_cache_del");
        let key = Self::schema_cache_key(data_source_id);
        let mut conn = match self.redis_pool.get().await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(error = %e, "获取 Redis 连接失败，跳过删缓存");
                return;
            }
        };
        if let Err(e) = conn.del::<_, ()>(&key).await {
            tracing::warn!(error = %e, key, "删除 Redis 元数据缓存失败");
        }
    }

    #[tracing::instrument(skip(self), fields(data_source_id = data_source_id))]
    pub async fn get_schema(
        &self,
        data_source_id: i64,
    ) -> AppResult<DataSourceSchemaVo> {
        let cache_key = Self::schema_cache_key(data_source_id);

        // 1. 优先读 Redis 缓存
        if let Some(vo) = self.read_schema_cache(&cache_key).await {
            tracing::info!(data_source_id, "命中 Redis 元数据缓存");
            return Ok(vo);
        }

        // 2. 未命中 → 查 MySQL
        let vo = self.load_schema_from_db(data_source_id).await?;

        // 3. 回填缓存（失败不致命）
        self.write_schema_cache(&cache_key, &vo).await;

        Ok(vo)
    }

    /// 从 MySQL 组装数据源的表/列元数据，不经过缓存
    async fn load_schema_from_db(
        &self,
        data_source_id: i64,
    ) -> AppResult<DataSourceSchemaVo> {
        let tables = self
            .table_meta_repo
            .select_by_data_source_id(&data_source_id)
            .await?;

        let table_meta_ids: Vec<i64> =
            tables.iter().filter_map(|t| t.table_meta_id).collect();

        // 一次性查出该数据源下所有列，再按 table_meta_id 在内存里分组，
        // 避免按表逐次查询（N+1）。
        let all_columns = self
            .column_meta_repo
            .select_by_table_meta_ids(&table_meta_ids)
            .await?;

        let mut columns_by_table: HashMap<i64, Vec<DataSourceColumnSchemaVo>> =
            HashMap::new();
        for c in all_columns {
            columns_by_table.entry(c.table_meta_id).or_default().push(
                DataSourceColumnSchemaVo {
                    column_meta_id: c.column_meta_id.unwrap_or_default(),
                    column_name: c.column_name,
                    comment_label: c.comment_label,
                    field_type: c.field_type,
                    format_pattern: c.format_pattern,
                    field_desc: c.field_desc,
                    source_physical_type: c.source_physical_type,
                    ordinal_position: c.ordinal_position,
                    is_primary_key: c.is_primary_key,
                },
            );
        }

        let mut out_tables = Vec::with_capacity(tables.len());
        for tm in tables {
            let tid = tm.table_meta_id.ok_or_else(|| {
                AppError::Internal("表元数据ID为空".to_string())
            })?;
            out_tables.push(DataSourceTableSchemaVo {
                table_meta_id: tid,
                table_name: tm.table_name,
                comment_label: tm.comment_label,
                table_desc: tm.table_desc,
                columns: columns_by_table.remove(&tid).unwrap_or_default(),
            });
        }
        tracing::info!(
            data_source_id,
            table_count = out_tables.len(),
            "从 MySQL 加载数据源元数据成功"
        );
        Ok(DataSourceSchemaVo {
            data_source_id,
            tables: out_tables,
        })
    }

    #[tracing::instrument(skip(self, current_user), fields(data_source_id = data_source_id, username = %current_user.username))]
    pub async fn refresh_schema(
        &self,
        data_source_id: i64,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let data_source =
            self.get_decrypted_data_source(data_source_id).await?;
        let db_type = data_source.db_type.to_lowercase();
        tracing::info!(
            data_source_id,
            db_type = %db_type,
            "开始刷新数据源元数据"
        );
        let introspected = match db_type.as_str() {
            "mysql" => {
                let ds = data_source.clone();
                tokio::time::timeout(
                    Duration::from_secs(600),
                    tokio::task::spawn_blocking(move || introspect_mysql(&ds)),
                )
                .await
                .map_err(|_| {
                    AppError::Internal("刷新 MySQL 元数据超时".to_string())
                })?
                .map_err(|e| {
                    AppError::Internal(format!("任务执行失败: {}", e))
                })?
                .map_err(|e| AppError::Internal(e.to_string()))?
            }
            "postgresql" | "postgres" => {
                let ds = data_source.clone();
                tokio::time::timeout(
                    Duration::from_secs(600),
                    tokio::task::spawn_blocking(move || {
                        introspect_postgresql(&ds)
                    }),
                )
                .await
                .map_err(|_| {
                    AppError::Internal("刷新 PostgreSQL 元数据超时".to_string())
                })?
                .map_err(|e| {
                    AppError::Internal(format!("任务执行失败: {}", e))
                })?
                .map_err(|e| AppError::Internal(e.to_string()))?
            }
            _ => {
                return Err(AppError::Internal(format!(
                    "暂不支持刷新该类型的数据源: {}",
                    data_source.db_type
                )));
            }
        };

        self.table_meta_repo
            .delete_all_for_data_source(data_source_id)
            .await?;

        let uid = current_user.get_user_id();

        // 1. 批量插入表元数据
        let table_metas: Vec<DataSourceTableMeta> = introspected
            .iter()
            .map(|t| DataSourceTableMeta {
                table_meta_id: None,
                data_source_id,
                table_name: t.table_name.clone(),
                comment_label: t.comment_label.clone(),
                table_desc: None,
                base_entity: BaseEntity::new(uid),
            })
            .collect();
        self.table_meta_repo.insert_batch(&table_metas).await?;

        // 2. 查询获取 table_meta_id 映射
        let saved_tables = self
            .table_meta_repo
            .select_by_data_source_id(&data_source_id)
            .await?;
        let table_id_map: HashMap<String, i64> = saved_tables
            .into_iter()
            .filter_map(|t| t.table_meta_id.map(|id| (t.table_name, id)))
            .collect();

        // 3. 批量插入列元数据
        let mut column_metas = Vec::new();
        for t in introspected {
            let tid =
                table_id_map.get(&t.table_name).copied().ok_or_else(|| {
                    AppError::Internal(format!(
                        "表 '{}' 的元数据ID未找到",
                        t.table_name
                    ))
                })?;
            for c in t.columns {
                column_metas.push(DataSourceColumnMeta {
                    column_meta_id: None,
                    table_meta_id: tid,
                    column_name: c.column_name,
                    comment_label: c.comment_label,
                    field_type: c.field_type,
                    format_pattern: None,
                    field_desc: None,
                    source_physical_type: c.source_physical_type,
                    ordinal_position: c.ordinal_position,
                    is_primary_key: c.is_primary_key as i32,
                    base_entity: BaseEntity::new(uid),
                });
            }
        }
        self.column_meta_repo.insert_batch(&column_metas).await?;

        tracing::info!(
            data_source_id,
            db_type = %db_type,
            table_count = table_metas.len(),
            column_count = column_metas.len(),
            "刷新数据源元数据成功"
        );

        // 用最新元数据覆盖 Redis 缓存（失败不致命，下次 get_schema 会回填）
        match self.load_schema_from_db(data_source_id).await {
            Ok(vo) => {
                self.write_schema_cache(
                    &Self::schema_cache_key(data_source_id),
                    &vo,
                )
                .await;
            }
            Err(e) => tracing::warn!(
                data_source_id,
                error = %e,
                "刷新后重建元数据缓存失败"
            ),
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn list(&self) -> AppResult<Vec<DataSourceListVo>> {
        let result = self.data_source_repo.select_all().await?;
        let vo_list: Vec<DataSourceListVo> = result
            .into_iter()
            .map(|entity| entity.to_list_vo())
            .collect();
        tracing::info!(count = vo_list.len(), "查询数据源列表成功");
        Ok(vo_list)
    }

    #[tracing::instrument(skip(self, query))]
    pub async fn page(
        &self,
        query: DataSourcePageQuery,
    ) -> AppResult<PageResult<DataSourceListVo>> {
        let (current, size, _offset) = normalize_paging(&query.page);
        tracing::info!(current, size, "分页查询数据源");
        let allowed = [
            ("dataSourceId", "data_source_id"),
            ("code", "code"),
            ("name", "name"),
            ("dbType", "db_type"),
            ("createdDate", "created_date"),
        ];
        let mut order_by =
            sanitize_sort(&query.page.sort, &query.page.order, &allowed);
        if order_by.is_none() {
            order_by = Some("data_source_id desc".to_string());
        }

        let order_str = order_by.as_deref().unwrap_or("data_source_id desc");
        let page = self
            .data_source_repo
            .select_page(&query, current as u64, size as u64, order_str)
            .await?;
        let total = page.total as i64;
        if total <= 0 {
            return Ok(PageResult::empty(current, size));
        }
        let pages = (total + size - 1) / size;
        tracing::info!(current, size, total, pages, "分页查询数据源成功");
        Ok(PageResult {
            records: page.records.into_iter().map(|e| e.to_list_vo()).collect(),
            total,
            pages,
            current,
            size,
        })
    }

    #[tracing::instrument(skip(self), fields(id = id))]
    pub async fn get_data_source_by_id(
        &self,
        id: i64,
    ) -> AppResult<DataSourceDetailVo> {
        let entity =
            self.data_source_repo.select_by_id(&id).await?.ok_or_else(
                || AppError::NotFound(format!("数据源不存在 (id={})", id)),
            )?;
        let vo = entity.to_detail_vo();
        tracing::info!(id, "查询数据源详情成功");
        Ok(vo)
    }

    /// 内部连接用：带解密明文密码的域对象。只许服务端流转，
    /// 不能作为 HTTP 响应返回。
    #[tracing::instrument(skip(self), fields(id = id))]
    pub async fn get_decrypted_data_source(
        &self,
        id: i64,
    ) -> AppResult<DataSource> {
        let entity =
            self.data_source_repo.select_by_id(&id).await?.ok_or_else(
                || AppError::NotFound(format!("数据源不存在 (id={})", id)),
            )?;
        entity
            .to_decrypted(self.encryption_key.as_bytes())
            .map_err(|e| {
                AppError::Internal(format!("解密数据源密码失败: {}", e))
            })
    }

    #[tracing::instrument(skip(self, bo, current_user), fields(username = %current_user.username))]
    pub async fn insert_by_bo(
        &self,
        bo: DataSourceCreateBo,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        if self
            .data_source_repo
            .find_by_code(&bo.code)
            .await?
            .is_some()
        {
            return Err(AppError::Internal(format!(
                "code '{}' 已被使用",
                bo.code
            )));
        }
        let entity = DataSource::from_create_bo(
            bo,
            current_user,
            self.encryption_key.as_bytes(),
        )
        .map_err(|e| {
            tracing::error!(error = %e, "加密数据源密码失败");
            AppError::Internal(format!("加密数据源密码失败: {}", e))
        })?;
        self.data_source_repo.insert(&entity).await?;
        tracing::info!(data_source_id = ?entity.data_source_id, "创建数据源成功");
        Ok(())
    }

    #[tracing::instrument(skip(self, bo, current_user), fields(username = %current_user.username))]
    pub async fn update_by_bo(
        &self,
        bo: DataSourceUpdateBo,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let data_source_id = bo
            .data_source_id
            .ok_or_else(|| AppError::BadRequest("数据源ID为空".to_string()))?;
        if self
            .data_source_repo
            .find_by_code_excluding(&bo.code, &data_source_id)
            .await?
            .is_some()
        {
            return Err(AppError::BadRequest(format!(
                "code '{}' 已被使用",
                bo.code
            )));
        }
        let mut entity = self
            .data_source_repo
            .select_by_id(&data_source_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "数据源不存在 (id={})",
                    data_source_id
                ))
            })?;
        entity
            .from_update_bo(bo, current_user, self.encryption_key.as_bytes())
            .map_err(|e| {
                tracing::error!(error = %e, data_source_id, "加密数据源密码失败");
                AppError::Internal(format!("加密数据源密码失败: {}", e))
            })?;
        self.data_source_repo
            .update_by_id(&entity, &data_source_id)
            .await?;
        tracing::info!(data_source_id, "更新数据源成功");
        Ok(())
    }

    #[tracing::instrument(skip(self, current_user), fields(id = id, username = %current_user.username))]
    pub async fn delete_by_id(
        &self,
        id: i64,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let mut entity =
            self.data_source_repo.select_by_id(&id).await?.ok_or_else(
                || AppError::NotFound(format!("数据源不存在 (id={})", id)),
            )?;

        entity.base_entity.delete(current_user.get_user_id());

        self.data_source_repo.update_by_id(&entity, &id).await?;
        tracing::info!(id, "软删除数据源成功");
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(id = id))]
    pub async fn true_delete_by_id(&self, id: i64) -> AppResult<()> {
        self.data_source_repo.delete_by_id(&id).await?;
        // 元数据缓存无过期，删除数据源时一并清掉，避免留下永久孤儿键
        self.evict_schema_cache(id).await;
        tracing::info!(id, "彻底删除数据源成功");
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(id = id))]
    pub async fn test_data_source(&self, id: i64) -> AppResult<String> {
        let data_source = self.get_decrypted_data_source(id).await?;
        tracing::info!(
            id,
            db_type = %data_source.db_type,
            host = %data_source.db_host,
            port = data_source.db_port,
            "开始测试数据源连接"
        );
        let (success, message) = test_data_source(&data_source).await;
        if success {
            tracing::info!(
                id,
                db_type = %data_source.db_type,
                host = %data_source.db_host,
                port = data_source.db_port,
                "数据源连接测试成功"
            );
            Ok(message)
        } else {
            tracing::warn!(
                id,
                db_type = %data_source.db_type,
                host = %data_source.db_host,
                port = data_source.db_port,
                message = %message,
                "数据源连接测试失败"
            );
            Err(AppError::Internal(message))
        }
    }
}
