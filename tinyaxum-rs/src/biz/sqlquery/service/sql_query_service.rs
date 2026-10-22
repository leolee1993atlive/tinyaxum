use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex},
    time::{Duration, Instant},
};

use duckdb::Connection;
use regex::Regex;
use serde_json::{Map, Value};

use crate::{
    app::Infrastructure,
    biz::{
        datasource::{
            model::data_source_schema_vo::{
                DataSourceSchemaVo, DataSourceTableSchemaVo,
            },
            service::data_source_service::DataSourceService,
        },
        sqlquery::{
            model::{
                sql_query::{
                    SqlQuery, SqlQueryCreateBo, SqlQueryDetailVo,
                    SqlQueryListVo, SqlQueryPageQuery, SqlQueryUpdateBo,
                },
                sql_query_param_field::SqlQueryParamField,
                sql_query_result_field::SqlQueryResultField,
            },
            repository::{
                sql_query_param_field_repo::SqlQueryParamFieldRepository,
                sql_query_repo::SqlQueryRepository,
                sql_query_result_field_repo::SqlQueryResultFieldRepository,
                sql_snippet_repo::SqlSnippetRepository,
            },
        },
    },
    common::{
        model::field_semantic_type::FieldSemanticType,
        vo::{
            page::{PageResult, normalize_paging, sanitize_sort},
            sql_run::{SqlResultColumnVo, SqlRunResultVo},
        },
    },
    error::error::{AppError, AppResult},
    middleware::extractors::CurrentUser,
    util::{
        duckdb_catalog::{DuckdbCatalog, TableMap},
        sql_parser::SqlParser,
    },
};

struct SnippetCache {
    data: HashMap<String, String>,
    loaded_at: Instant,
}

#[derive(Clone)]
pub struct SqlQueryService {
    sql_query_repo: SqlQueryRepository,
    sql_snippet_repo: SqlSnippetRepository,
    result_field_repo: SqlQueryResultFieldRepository,
    param_field_repo: SqlQueryParamFieldRepository,
    /// 每个数据源的同步产物根目录：`<duckdb_data_dir>/src_<code>/<table>.duckdb`
    /// 通过 `DuckdbCatalog` 统一管理 + 缓存目录扫描结果。
    duckdb_catalog: Arc<DuckdbCatalog>,
    data_source_service: DataSourceService,
    sql_parser: Arc<SqlParser>,
    snippet_cache: Arc<Mutex<Option<SnippetCache>>>,
}

impl SqlQueryService {
    pub fn new(infra: &Infrastructure) -> AppResult<Self> {
        Ok(SqlQueryService {
            sql_query_repo: SqlQueryRepository::new(infra.db.clone()),
            sql_snippet_repo: SqlSnippetRepository::new(infra.db.clone()),
            result_field_repo: SqlQueryResultFieldRepository::new(
                infra.db.clone(),
            ),
            param_field_repo: SqlQueryParamFieldRepository::new(
                infra.db.clone(),
            ),
            duckdb_catalog: Arc::new(DuckdbCatalog::new(
                infra.duckdb_data_dir.clone(),
                Duration::from_secs(30),
            )),
            data_source_service: DataSourceService::new(infra),
            sql_parser: Arc::new(SqlParser::new()?),
            snippet_cache: Arc::new(Mutex::new(None)),
        })
    }

    /// 仅保留 `table_name`(小写)出现在 DuckDB 已导出清单中的表。
    /// catalog 的 key 是小写表名,故此处做大小写不敏感匹配。
    fn filter_tables_to_exported(
        tables: Vec<DataSourceTableSchemaVo>,
        available: &TableMap,
    ) -> Vec<DataSourceTableSchemaVo> {
        tables
            .into_iter()
            .filter(|t| available.contains_key(&t.table_name.to_lowercase()))
            .collect()
    }

    /// SQL 编辑器专用:只返回该数据源下"实际已导出到 DuckDB"的表元数据。
    ///
    /// 与 `DataSourceService::get_schema` 不同——后者返回源库全部表(供建传输
    /// 任务选表)。这里以 `DuckdbCatalog` 扫描到的 `.duckdb` 文件清单为准过滤,
    /// 因为查询/预览不会回落到源库,未导出的表列出来也查不了。
    #[tracing::instrument(skip(self), fields(data_source_id = data_source_id))]
    pub async fn get_query_schema(
        &self,
        data_source_id: i64,
    ) -> AppResult<DataSourceSchemaVo> {
        // 两次调用各取所需:get_schema 拿全量表元数据(带缓存),
        // get_data_source_by_id 仅为拿 source_code 解析 DuckDB 目录。
        let full = self.data_source_service.get_schema(data_source_id).await?;
        let ds = self
            .data_source_service
            .get_data_source_by_id(data_source_id)
            .await?;
        let source_code = ds.code;

        // catalog.lookup 在该数据源从未同步过(src_<code> 目录不存在)时会报错。
        // 这里有意把所有 lookup 错误都按"无已导出表"处理:SQL 编辑器只需一个空
        // 面板,不该因元数据目录问题弹 500。
        let available = match self.duckdb_catalog.lookup(&source_code) {
            Ok(map) => map,
            Err(e) => {
                tracing::debug!(
                    data_source_id,
                    source_code = %source_code,
                    error = %e,
                    "DuckDB 目录不可用,按无已导出表处理"
                );
                return Ok(DataSourceSchemaVo {
                    data_source_id,
                    tables: Vec::new(),
                });
            }
        };

        let tables =
            Self::filter_tables_to_exported(full.tables, available.as_ref());
        tracing::info!(
            data_source_id,
            table_count = tables.len(),
            "返回 SQL 编辑器已导出表元数据"
        );
        Ok(DataSourceSchemaVo {
            data_source_id,
            tables,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn list(&self) -> AppResult<Vec<SqlQueryListVo>> {
        let result = self.sql_query_repo.select_all_lite().await?;
        tracing::info!(count = result.len(), "查询 SQL 查询列表成功");
        Ok(result
            .into_iter()
            .map(|entity| entity.to_list_vo())
            .collect())
    }

    #[tracing::instrument(skip(self, query))]
    pub async fn page(
        &self,
        query: SqlQueryPageQuery,
    ) -> AppResult<PageResult<SqlQueryListVo>> {
        let (current, size, _offset) = normalize_paging(&query.page);
        tracing::info!(
            current = current,
            size = size,
            keyword = ?query.keyword,
            "SQL 查询分页查询"
        );
        let allowed = [
            ("sqlQueryId", "sql_query_id"),
            ("sqlQueryName", "sql_query_name"),
            ("dataSourceId", "data_source_id"),
            ("isCollection", "is_collection"),
            ("createdDate", "created_date"),
        ];
        let mut order_by =
            sanitize_sort(&query.page.sort, &query.page.order, &allowed);
        if order_by.is_none() {
            order_by = Some("sql_query_id desc".to_string());
        }

        let order_str = order_by.as_deref().unwrap_or("sql_query_id desc");
        let page = self
            .sql_query_repo
            .select_page(&query, current as u64, size as u64, order_str)
            .await
            .inspect_err(|e| {
                tracing::error!(error = %e, "SQL 查询分页数据获取失败");
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
            "SQL 查询分页查询完成"
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
    pub async fn get_sql_query_by_id(
        &self,
        id: i64,
    ) -> AppResult<SqlQueryDetailVo> {
        let entity =
            self.sql_query_repo
                .select_by_id(&id)
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!("SQL 查询不存在 (id={})", id))
                })?;
        tracing::info!(id, "根据ID查询 SQL 查询成功");
        let mut vo = entity.to_detail_vo();
        let (rfs, pfs) = tokio::join!(
            self.result_field_repo.select_by_sql_query_id(&id),
            self.param_field_repo.select_by_sql_query_id(&id),
        );
        match rfs {
            Ok(rows) => {
                vo.result_fields = rows.iter().map(|r| r.to_vo()).collect();
            }
            Err(e) => {
                tracing::error!(error = %e, id, "读取结果字段元数据失败");
            }
        }
        match pfs {
            Ok(rows) => {
                vo.param_fields = rows.iter().map(|p| p.to_vo()).collect();
            }
            Err(e) => {
                tracing::error!(error = %e, id, "读取参数字段元数据失败");
            }
        }
        Ok(vo)
    }

    #[tracing::instrument(skip(self, bo, current_user), fields(username = %current_user.username))]
    pub async fn insert_by_bo(
        &self,
        bo: SqlQueryCreateBo,
        current_user: &CurrentUser,
    ) -> AppResult<i64> {
        let user_id = current_user.get_user_id();
        if let Some(existing) = self
            .sql_query_repo
            .select_latest_by_name_for_user(&bo.sql_query_name, &user_id)
            .await
            .ok()
            .flatten()
        {
            tracing::warn!(
                sql_query_name = %bo.sql_query_name,
                existing_id = ?existing.sql_query_id,
                "同名查询已存在，返回已有 ID"
            );
            return Ok(existing.sql_query_id.unwrap_or(0));
        }

        let entity = SqlQuery::from_create_bo(bo, current_user);
        let id = self.sql_query_repo.insert(&entity).await?;
        tracing::info!(sql_query_id = id, "创建 SQL 查询成功");
        if let Err(e) = self
            .sync_param_fields(id, &entity.sql_query_content, current_user)
            .await
        {
            tracing::error!(error = %e, sql_query_id = id, "同步参数字段失败");
        }
        Ok(id)
    }

    #[tracing::instrument(skip(self, bo, current_user), fields(username = %current_user.username))]
    pub async fn update_by_bo(
        &self,
        bo: SqlQueryUpdateBo,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let sql_query_id = bo
            .sql_query_id
            .ok_or_else(|| AppError::BadRequest("查询ID为空".to_string()))?;
        let mut entity = self
            .sql_query_repo
            .select_by_id(&sql_query_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "SQL 查询不存在 (id={})",
                    sql_query_id
                ))
            })?;

        entity.from_update_bo(bo, current_user);
        let content = entity.sql_query_content.clone();
        self.sql_query_repo
            .update_by_id(&entity, &sql_query_id)
            .await?;
        tracing::info!(sql_query_id, "更新 SQL 查询成功");
        if let Err(e) = self
            .sync_param_fields(sql_query_id, &content, current_user)
            .await
        {
            tracing::error!(
                error = %e,
                sql_query_id,
                "同步参数字段失败"
            );
        }
        Ok(())
    }

    #[tracing::instrument(skip(self, viz_config, current_user), fields(id = id, username = %current_user.username))]
    pub async fn update_viz_config(
        &self,
        id: i64,
        viz_config: Option<Value>,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let mut entity = self
            .sql_query_repo
            .select_by_id(&id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("SQL 查询不存在 (id={})", id))
            })?;

        entity.viz_config = viz_config;
        entity.base_entity.update(current_user.get_user_id());

        self.sql_query_repo.update_by_id(&entity, &id).await?;
        tracing::info!(id, "更新可视化配置成功");
        Ok(())
    }

    #[tracing::instrument(skip(self, current_user), fields(id = id, username = %current_user.username))]
    pub async fn delete_by_id(
        &self,
        id: i64,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let mut entity = self
            .sql_query_repo
            .select_by_id(&id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("SQL 查询不存在 (id={})", id))
            })?;

        entity.base_entity.delete(current_user.get_user_id());

        self.sql_query_repo.update_by_id(&entity, &id).await?;
        tracing::info!(id, "软删除 SQL 查询成功");
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(id = id))]
    pub async fn true_delete_by_id(&self, id: i64) -> AppResult<()> {
        self.sql_query_repo.delete_by_id(&id).await?;
        tracing::info!(id, "彻底删除 SQL 查询成功");
        Ok(())
    }

    #[tracing::instrument(skip(self, sql_content, current_user), fields(sql_query_id = sql_query_id))]
    async fn sync_param_fields(
        &self,
        sql_query_id: i64,
        sql_content: &str,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        // 先展开片段再提取参数名：参数可能只出现在片段内容里，
        // 只扫查询原文会导致这类参数漏注册、甚至把手工补的行误删
        let snippet_map = self.get_snippet_map().await?;
        let expanded =
            self.sql_parser.expand_snippets(sql_content, &snippet_map)?;
        let names = self.sql_parser.extract_parameter_names(&expanded);
        let name_set: HashSet<String> = names.iter().cloned().collect();
        let existing = self
            .param_field_repo
            .select_by_sql_query_id(&sql_query_id)
            .await
            .inspect_err(|e| {
                tracing::error!(
                    error = %e,
                    sql_query_id,
                    "查询已有参数字段失败"
                );
            })?;
        let uid = current_user.get_user_id();

        for row in &existing {
            if name_set.contains(&row.field_name) {
                continue;
            }
            let Some(pid) = row.param_field_id else {
                continue;
            };
            let mut ent = row.clone();
            ent.base_entity.delete(uid);
            self.param_field_repo
                .update_by_id(&ent, &pid)
                .await
                .inspect_err(|e| {
                    tracing::error!(
                        error = %e,
                        param_field_id = pid,
                        "软删除参数字段失败"
                    );
                })?;
        }

        // 已存在且仍然活跃（未软删除、且本次未被删除）的参数名
        let existing_names: HashSet<String> = existing
            .iter()
            .filter(|r| {
                r.base_entity.deleted_by.is_none()
                    && name_set.contains(&r.field_name)
            })
            .map(|r| r.field_name.clone())
            .collect();

        let default_ft = FieldSemanticType::Text.as_str().to_string();
        for n in names {
            if existing_names.contains(&n) {
                continue;
            }
            let p = SqlQueryParamField {
                param_field_id: None,
                sql_query_id,
                field_name: n,
                comment_label: None,
                field_type: default_ft.clone(),
                format_pattern: None,
                field_desc: None,
                required_flag: 0,
                base_entity: crate::common::model::entity::BaseEntity::new(uid),
            };
            self.param_field_repo.insert(&p).await.inspect_err(|e| {
                tracing::error!(
                    error = %e,
                    sql_query_id,
                    "插入参数字段失败"
                );
            })?;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(sql_query_id = sql_query_id))]
    async fn merge_result_fields_from_run(
        &self,
        sql_query_id: i64,
        column_names: &[String],
        user_id: i64,
    ) -> AppResult<()> {
        let existing = self
            .result_field_repo
            .select_by_sql_query_id(&sql_query_id)
            .await
            .inspect_err(|e| {
                tracing::error!(
                    error = %e,
                    sql_query_id,
                    "查询已有结果字段失败"
                );
            })?;
        let existing_names: HashSet<String> =
            existing.iter().map(|e| e.field_name.clone()).collect();

        let default_ft = FieldSemanticType::Text.as_str().to_string();
        for name in column_names {
            if existing_names.contains(name) {
                continue;
            }
            let row = SqlQueryResultField {
                result_field_id: None,
                sql_query_id,
                field_name: name.clone(),
                comment_label: None,
                field_type: default_ft.clone(),
                format_pattern: None,
                field_desc: None,
                base_entity: crate::common::model::entity::BaseEntity::new(
                    user_id,
                ),
            };
            self.result_field_repo.insert(&row).await.inspect_err(|e| {
                tracing::error!(
                    error = %e,
                    sql_query_id,
                    field_name = name,
                    "插入结果字段失败"
                );
            })?;
        }

        let mut ent =
            match self.sql_query_repo.select_by_id(&sql_query_id).await {
                Ok(Some(e)) => e,
                Ok(None) => return Ok(()),
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        sql_query_id,
                        "查询 SQL 查询实体失败"
                    );
                    return Err(AppError::Internal(e.to_string()));
                }
            };
        ent.result_meta_updated_at = Some(chrono::Utc::now());
        self.sql_query_repo
            .update_by_id(&ent, &sql_query_id)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    sql_query_id,
                    "更新结果字段元数据时间戳失败"
                );
                AppError::Internal(e.to_string())
            })?;
        Ok(())
    }

    /// 检查 SQL 中是否存在引号外的分号（多语句攻击特征）
    fn contains_semicolon_outside_quotes(sql: &str) -> bool {
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut in_backtick = false;
        let mut escaped = false;

        for ch in sql.chars() {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            match ch {
                '\'' if !in_double_quote && !in_backtick => {
                    in_single_quote = !in_single_quote;
                }
                '"' if !in_single_quote && !in_backtick => {
                    in_double_quote = !in_double_quote;
                }
                '`' if !in_single_quote && !in_double_quote => {
                    in_backtick = !in_backtick;
                }
                ';' if !in_single_quote && !in_double_quote && !in_backtick => {
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    fn apply_paging(
        sql: &str,
        current: i64,
        size: i64,
    ) -> (String, Option<String>) {
        static LIMIT_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
            regex::Regex::new(r"(?i)\s+LIMIT\s+\d+(\s*OFFSET\s+\d+)?\s*$")
                .expect("apply_paging LIMIT 正则常量编译失败")
        });

        // 剥掉首尾空白和尾部连续的分号——SQL 客户端给出的 SQL 末尾常带 `;`，
        // 这是合法语法，但会破坏分页包装的子查询语义。
        let mut cleaned = sql.trim();
        loop {
            let next = cleaned.trim_end_matches(';').trim_end();
            if next.len() == cleaned.len() {
                break;
            }
            cleaned = next;
        }

        if cleaned.is_empty() {
            return (sql.to_string(), None);
        }

        // size<=0 表示调用方明确要求不分页（如公开 API 未传分页参数时返回全量），
        // 不做 LIMIT 包装；计数 SQL 一并省略，总数由返回行数兜底。
        if size <= 0 {
            return (sql.to_string(), None);
        }

        // 剥完尾分号后仍有引号外的分号 = 真正的多语句 SQL，拒绝包装
        if Self::contains_semicolon_outside_quotes(cleaned) {
            tracing::error!(
                "检测到潜在的多语句 SQL，拒绝添加分页包装以防止 SQL 注入"
            );
            return (sql.to_string(), None);
        }

        if LIMIT_RE.is_match(cleaned) {
            return (sql.to_string(), None);
        }

        let offset = (current - 1).max(0) * size;
        let paged_sql = format!(
            "SELECT * FROM ({cleaned}) AS _paged_query LIMIT {size} OFFSET {offset}"
        );
        let count_sql = format!(
            "SELECT COUNT(*) AS _total FROM ({cleaned}) AS _count_query"
        );
        (paged_sql, Some(count_sql))
    }

    /// 在 DuckDB 上执行计数查询。
    ///
    /// 每次调用都新建一个内存 DuckDB 连接，按需 ATTACH SQL 中真正引用到的表文件
    /// （`<duckdb_data_dir>/src_<code>/<table>.duckdb`）。开销与"数据源总表数"无关，
    /// 仅与"本次查询引用的表数"线性相关。
    #[tracing::instrument(skip(self, sql))]
    async fn count_on_duckdb(
        &self,
        sql: String,
        source_code: String,
    ) -> AppResult<i64> {
        let catalog = self.duckdb_catalog.clone();

        tokio::time::timeout(
            Duration::from_secs(60),
            tokio::task::spawn_blocking(move || {
                let _timer = crate::util::metrics::DuckDbQueryTimer::new();
                let conn = open_query_connection(&catalog, &source_code, &sql)?;

                let count: i64 = conn
                    .query_row(sql.as_str(), [], |row| row.get(0))
                    .map_err(|e| {
                        tracing::error!(
                            error = %e,
                            sql = %sql,
                            "DuckDB 计数查询失败"
                        );
                        AppError::Internal(format!(
                            "DuckDB 计数查询失败: {}",
                            crate::error::localizer::translate_raw(
                                &e.to_string()
                            ),
                        ))
                    })?;

                Ok(count)
            }),
        )
        .await
        .map_err(|_| {
            tracing::error!("DuckDB 计数查询超时（60秒）");
            AppError::Internal(
                "DuckDB 计数查询超时，请优化 SQL 或减少数据量".to_string(),
            )
        })?
        .map_err(|e| {
            tracing::error!(error = %e, "DuckDB 计数任务执行失败");
            AppError::Internal(format!("DuckDB 计数任务执行失败: {}", e))
        })?
    }

    /// 解析 SQL 并展开 snippet
    #[tracing::instrument(skip(self, sql_content, params))]
    async fn parse_sql(
        &self,
        sql_content: &str,
        params: &Option<HashMap<String, String>>,
    ) -> AppResult<String> {
        let snippet_map = self.get_snippet_map().await?;
        self.sql_parser
            .parse(sql_content, params, &snippet_map)
            .map_err(|e| {
                tracing::error!(error = %e, "解析SQL失败");
                AppError::Internal(format!("解析SQL失败: {:?}", e))
            })
    }

    async fn get_snippet_map(
        &self,
    ) -> AppResult<Option<HashMap<String, String>>> {
        const TTL: Duration = Duration::from_secs(60);

        {
            let guard = self.snippet_cache.lock().unwrap();
            if let Some(ref cache) = *guard
                && cache.loaded_at.elapsed() < TTL
            {
                if cache.data.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(cache.data.clone()));
            }
        }

        let snippets =
            self.sql_snippet_repo.select_all().await.inspect_err(|e| {
                tracing::error!(error = %e, "查询 SQL 片段列表失败");
            })?;

        let map: HashMap<String, String> = snippets
            .iter()
            .map(|s| {
                (s.sql_snippet_name.clone(), s.sql_snippet_content.clone())
            })
            .collect();

        let result = if map.is_empty() {
            None
        } else {
            Some(map.clone())
        };

        {
            let mut guard = self.snippet_cache.lock().unwrap();
            *guard = Some(SnippetCache {
                data: map,
                loaded_at: Instant::now(),
            });
        }

        Ok(result)
    }

    /// 在 DuckDB 上执行 SQL。
    ///
    /// 每次调用都新建一个内存 DuckDB 连接，按需 ATTACH SQL 中真正引用到的表文件
    /// （`<duckdb_data_dir>/src_<code>/<table>.duckdb`）。
    #[tracing::instrument(skip(self, sql, source_code))]
    async fn execute_on_duckdb(
        &self,
        sql: String,
        source_code: String,
    ) -> AppResult<SqlRunResultVo> {
        let catalog = self.duckdb_catalog.clone();

        tokio::time::timeout(
            Duration::from_secs(60),
            tokio::task::spawn_blocking(move || {
            let _timer = crate::util::metrics::DuckDbQueryTimer::new();
            let conn = open_query_connection(
                &catalog,
                &source_code,
                &sql,
            )?;

            let mut stmt = conn.prepare(sql.as_str()).map_err(|e| {
                tracing::error!(error = %e, sql = %sql, "DuckDB 预处理 SQL 失败");
                AppError::Internal(format!(
                    "DuckDB 预处理 SQL 失败: {}",
                    crate::error::localizer::translate_raw(&e.to_string()),
                ))
            })?;

            let mut col_count = 0usize;
            let mut rows_raw: Vec<Vec<Value>> = Vec::new();
            {
                let mut rows_iter = stmt.query([]).map_err(|e| {
                    tracing::error!(error = %e, sql = %sql, "DuckDB 执行查询失败");
                    AppError::Internal(format!(
                        "DuckDB 执行查询失败: {}",
                        crate::error::localizer::translate_raw(&e.to_string()),
                    ))
                })?;
                while let Some(row) = rows_iter.next().map_err(|e| {
                    tracing::error!(error = %e, "DuckDB 迭代行失败");
                    AppError::Internal(format!(
                        "DuckDB 迭代行失败: {}",
                        crate::error::localizer::translate_raw(&e.to_string()),
                    ))
                })? {
                    if col_count == 0 {
                        while row.get_ref(col_count).is_ok() {
                            col_count += 1;
                        }
                    }
                    let mut row_vals = Vec::with_capacity(col_count);
                    for idx in 0..col_count {
                        let json_val = row.get_ref(idx).map_err(|e| {
                            tracing::error!(
                                error = %e,
                                col_index = idx,
                                "DuckDB 读取列值失败"
                            );
                            AppError::Internal(format!(
                                "DuckDB 读取列值失败: {}",
                                crate::error::localizer::translate_raw(
                                    &e.to_string()
                                ),
                            ))
                        }).map(crate::util::duckdb_util::value_ref_to_json)?;
                        row_vals.push(json_val);
                    }
                    rows_raw.push(row_vals);
                }
            }

            let cols: Vec<String> =
                stmt.column_names().iter().map(|s| s.to_string()).collect();

            let n = stmt.column_count();
            let mut column_infos = Vec::with_capacity(n);
            for i in 0..n {
                let name = stmt
                    .column_name(i)
                    .map(|s| s.as_str().to_string())
                    .unwrap_or_default();
                let physical_type = format!("{:?}", stmt.column_type(i));
                column_infos.push(SqlResultColumnVo {
                    name,
                    physical_type,
                });
            }

            let row_count = rows_raw.len();
            let rows: Vec<Map<String, Value>> = rows_raw
                .into_iter()
                .map(|row_vals| {
                    cols.iter()
                        .zip(row_vals)
                        .map(|(name, val)| (name.clone(), val))
                        .collect()
                })
                .collect();

            tracing::info!(
                row_count = row_count,
                column_count = n,
                "DuckDB 查询执行成功"
            );

            Ok(SqlRunResultVo {
                columns: column_infos,
                rows,
                total: 0,
                current: 0,
                size: 0,
                pages: 0,
                is_collection: 0,
            })
        }))
        .await
        .map_err(|_| {
            tracing::error!("DuckDB 查询超时（60秒）");
            AppError::Internal(
                "DuckDB 查询超时，请优化 SQL 或减少数据量".to_string(),
            )
        })?
        .map_err(|e| {
            tracing::error!(error = %e, "DuckDB 任务执行失败");
            AppError::Internal(format!("DuckDB 任务执行失败: {}", e))
        })?
    }

    /// 执行 SQL 查询，根据 data_source_id 路由到不同数据库
    #[tracing::instrument(skip(self, sql_content, params), fields(data_source_id = data_source_id))]
    async fn execute_sql(
        &self,
        sql_content: &str,
        params: &Option<HashMap<String, String>>,
        data_source_id: i64,
        current: i64,
        size: i64,
        need_count: bool,
    ) -> AppResult<SqlRunResultVo> {
        let sql =
            self.parse_sql(sql_content, params).await.inspect_err(|e| {
                tracing::error!(
                    error = %e,
                    data_source_id,
                    "SQL 解析失败"
                );
            })?;

        let (paged_sql, count_sql) = Self::apply_paging(&sql, current, size);

        // 新架构下查询必须关联数据源：每个数据源的同步产物在
        // `<duckdb_data_dir>/src_<code>/*.duckdb`，没有数据源就没有可查表，
        // 也不再支持裸内存 DuckDB（避免 read_csv 任意路径读取的安全隐患）。
        if data_source_id <= 0 {
            tracing::error!(data_source_id, "查询未关联数据源");
            return Err(AppError::BadRequest(
                "查询必须关联数据源（data_source_id 必须 > 0）".to_string(),
            ));
        }
        let ds = self
            .data_source_service
            .get_data_source_by_id(data_source_id)
            .await
            .map_err(|_e| {
                tracing::error!(data_source_id, "数据源不存在");
                AppError::BadRequest(format!(
                    "数据源不存在: {}",
                    data_source_id
                ))
            })?;
        let source_code = ds.code.clone();

        tracing::info!(
            data_source_id,
            source_code = %source_code,
            "使用 DuckDB 执行 SQL"
        );

        let mut result = self
            .execute_on_duckdb(paged_sql, source_code.clone())
            .await?;

        let total = if need_count {
            match count_sql {
                Some(count_sql) => {
                    self.count_on_duckdb(count_sql, source_code).await?
                }
                None => result.rows.len() as i64,
            }
        } else {
            result.rows.len() as i64
        };

        let pages = if size <= 0 {
            0
        } else {
            (total + size - 1) / size
        };

        result.total = total;
        result.current = current;
        result.size = size;
        result.pages = pages;

        Ok(result)
    }

    #[tracing::instrument(skip(self, params), fields(id = id))]
    pub async fn run_sql_query(
        &self,
        id: i64,
        params: Option<HashMap<String, String>>,
        current_user_id: i64,
        current: i64,
        size: i64,
    ) -> AppResult<SqlRunResultVo> {
        let entity =
            self.sql_query_repo
                .select_by_id(&id)
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!("SQL 查询不存在 (id={})", id))
                })?;

        let query_sql = entity.sql_query_content;
        let ds_id = entity.data_source_id.unwrap_or(0);
        tracing::info!(id, data_source_id = ds_id, "开始执行 SQL 查询");
        let mut out = self
            .execute_sql(&query_sql, &params, ds_id, current, size, true)
            .await
            .inspect_err(|e| {
                tracing::error!(
                    error = %e,
                    id,
                    data_source_id = ds_id,
                    "SQL 查询执行失败"
                );
            })?;
        out.is_collection = entity.is_collection;
        let row_count = out.rows.len();
        let names: Vec<String> =
            out.columns.iter().map(|c| c.name.clone()).collect();
        if let Err(e) = self
            .merge_result_fields_from_run(id, &names, current_user_id)
            .await
        {
            tracing::error!(
                error = %e,
                id,
                "写入结果字段元数据失败"
            );
        }
        tracing::info!(
            id,
            data_source_id = ds_id,
            row_count = row_count,
            "SQL 查询执行成功"
        );
        Ok(out)
    }

    pub async fn run_for_dashboard(
        &self,
        id: i64,
        params: Option<HashMap<String, String>>,
        max_rows: i64,
    ) -> AppResult<SqlRunResultVo> {
        let entity =
            self.sql_query_repo
                .select_by_id(&id)
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!("SQL 查询不存在 (id={})", id))
                })?;
        let is_collection = entity.is_collection;
        let ds_id = entity.data_source_id.unwrap_or(0);
        let size = if is_collection == 0 {
            1
        } else {
            max_rows.max(1)
        };
        let mut out = self
            .execute_sql(
                &entity.sql_query_content,
                &params,
                ds_id,
                1,
                size,
                false,
            )
            .await
            .inspect_err(|e| {
                tracing::error!(error = %e, id, "首页查询现算失败");
            })?;
        out.is_collection = is_collection;
        Ok(out)
    }

    #[tracing::instrument(skip(self, sql_content, params), fields(data_source_id = data_source_id))]
    pub async fn run_adhoc_sql(
        &self,
        sql_content: String,
        params: Option<HashMap<String, String>>,
        data_source_id: i64,
        current: i64,
        size: i64,
    ) -> AppResult<SqlRunResultVo> {
        tracing::info!(
            data_source_id,
            sql_len = sql_content.len(),
            "开始执行 Adhoc SQL"
        );
        let result = self
            .execute_sql(
                &sql_content,
                &params,
                data_source_id,
                current,
                size,
                true,
            )
            .await
            .inspect_err(|e| {
                tracing::error!(
                    error = %e,
                    data_source_id,
                    "Adhoc SQL 执行失败"
                );
            })?;
        tracing::info!(
            data_source_id,
            row_count = result.rows.len(),
            "Adhoc SQL 执行成功"
        );
        Ok(result)
    }

    pub async fn validate_sql(
        &self,
        sql_content: String,
        data_source_id: i64,
    ) -> AppResult<Option<String>> {
        if data_source_id <= 0 {
            return Ok(None);
        }
        let ds = self
            .data_source_service
            .get_data_source_by_id(data_source_id)
            .await?;
        // 先展开片段再 EXPLAIN：片段承载查询主体，剥掉会把
        // `FROM {{snippet:x}}` 变成悬空 FROM；展开后才能校验真实 SQL。
        // 未同步的表会报 table not found，属于有信息量的校验结果。
        let snippet_map = self.get_snippet_map().await?;
        let expanded = self
            .sql_parser
            .expand_snippets(&sql_content, &snippet_map)?;
        let processed = preprocess_sql_for_explain(&expanded);
        let catalog = self.duckdb_catalog.clone();
        let source_code = ds.code.clone();

        let result = tokio::time::timeout(
            Duration::from_secs(10),
            tokio::task::spawn_blocking(move || {
                let conn = open_validate_connection(
                    &catalog,
                    &source_code,
                    &processed,
                )?;
                let explain_sql = format!("EXPLAIN {}", processed);
                match conn.execute(&explain_sql, []) {
                    Ok(_) => Ok(None),
                    Err(e) => Ok(Some(crate::error::localizer::translate_raw(
                        &e.to_string(),
                    ))),
                }
            }),
        )
        .await;

        match result {
            Ok(Ok(Ok(res))) => Ok(res),
            Ok(Ok(Err(e))) => Err(e),
            Ok(Err(_)) => {
                Err(AppError::Internal("校验 SQL 任务执行失败".to_string()))
            }
            Err(_) => Err(AppError::Internal("校验 SQL 超时".to_string())),
        }
    }
}

/// 为本次查询打开一个新的内存 DuckDB 连接，按需 ATTACH SQL 中引用到的表文件。
///
/// 流程：
/// 1. 通过 `DuckdbCatalog` 拿到该数据源的 `表名 → 路径` 映射（带 TTL 缓存）
/// 2. 用 word-boundary 正则扫描 SQL，把每个标识符候选去 catalog 里查；命中即 ATTACH
///    （比 SqlParser 抽 FROM/JOIN 更鲁棒：能覆盖逗号分隔多表、子查询、CTE 引用等）
/// 3. SET search_path 让用户 SQL 中未限定的表名能解析
///
/// 误命中（列名/字符串字面量恰好等于某表文件名）只会多 ATTACH 一个 READ_ONLY 库，
/// 实际查询用不到也无副作用。
fn open_query_connection(
    catalog: &DuckdbCatalog,
    source_code: &str,
    sql: &str,
) -> AppResult<Connection> {
    static IDENT_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"[A-Za-z_][A-Za-z_0-9]*(?:-[A-Za-z_0-9]+)*").unwrap()
    });

    let available = catalog.lookup(source_code)?;

    let mut to_attach: Vec<(String, PathBuf)> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for m in IDENT_RE.find_iter(sql) {
        let key = m.as_str().to_lowercase();
        if let Some((real, path)) = available.get(&key)
            && seen.insert(real.clone())
        {
            to_attach.push((real.clone(), path.clone()));
        }
    }

    if to_attach.is_empty() {
        let available_names: Vec<&str> =
            available.values().map(|(name, _)| name.as_str()).collect();
        tracing::warn!(
            source_code = %source_code,
            available_tables = ?available_names,
            "SQL 未匹配到任何已同步的 DuckDB 表文件"
        );
        return Err(AppError::Internal(format!(
            "查询未引用数据源 '{}' 下任何已同步表（已有同步表: {}）",
            source_code,
            if available_names.is_empty() {
                "无".to_string()
            } else {
                available_names.join(", ")
            },
        )));
    }

    let conn = Connection::open_in_memory().map_err(|e| {
        AppError::Internal(format!(
            "打开内存 DuckDB 连接失败: {}",
            crate::error::localizer::translate_raw(&e.to_string()),
        ))
    })?;

    for (table, path) in &to_attach {
        let path_str = path.to_str().ok_or_else(|| {
            AppError::Internal(format!(
                "DuckDB 文件路径含非 UTF-8 字符: {:?}",
                path
            ))
        })?;
        let escaped_path = path_str.replace('\'', "''");
        let attach_sql =
            format!("ATTACH '{}' AS \"{}\" (READ_ONLY)", escaped_path, table);
        conn.execute(&attach_sql, []).map_err(|e| {
            tracing::error!(
                error = %e,
                table = %table,
                path = %path.display(),
                "ATTACH 表文件失败"
            );
            AppError::Internal(format!(
                "ATTACH 表 '{}' 失败: {}",
                table,
                crate::error::localizer::translate_raw(&e.to_string()),
            ))
        })?;
    }

    // SET search_path 让未限定的 SELECT * FROM <table> 能解析到对应 attached db
    let search_path = to_attach
        .iter()
        .map(|(t, _)| format!("{}.main", t))
        .collect::<Vec<_>>()
        .join(",");
    conn.execute(&format!("SET search_path = '{}'", search_path), [])
        .map_err(|e| {
            AppError::Internal(format!(
                "设置 search_path 失败: {}",
                crate::error::localizer::translate_raw(&e.to_string()),
            ))
        })?;

    tracing::debug!(
        source_code = %source_code,
        attached = ?to_attach.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>(),
        "DuckDB 查询连接已就绪"
    );

    Ok(conn)
}

/// 为校验 SQL 打开一个内存 DuckDB 连接，按需 ATTACH 表文件。
/// 与 `open_query_connection` 的区别：不强制要求 SQL 引用到已同步表，
/// 允许纯语法校验（如 `SELECT 1`）。
fn open_validate_connection(
    catalog: &DuckdbCatalog,
    source_code: &str,
    sql: &str,
) -> AppResult<Connection> {
    static IDENT_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"[A-Za-z_][A-Za-z_0-9]*(?:-[A-Za-z_0-9]+)*").unwrap()
    });

    let available = catalog.lookup(source_code)?;

    let mut to_attach: Vec<(String, PathBuf)> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for m in IDENT_RE.find_iter(sql) {
        let key = m.as_str().to_lowercase();
        if let Some((real, path)) = available.get(&key)
            && seen.insert(real.clone())
        {
            to_attach.push((real.clone(), path.clone()));
        }
    }

    let conn = Connection::open_in_memory().map_err(|e| {
        AppError::Internal(format!(
            "打开内存 DuckDB 连接失败: {}",
            crate::error::localizer::translate_raw(&e.to_string()),
        ))
    })?;

    for (table, path) in &to_attach {
        let path_str = path.to_str().ok_or_else(|| {
            AppError::Internal(format!(
                "DuckDB 文件路径含非 UTF-8 字符: {:?}",
                path
            ))
        })?;
        let escaped_path = path_str.replace('\'', "''");
        let attach_sql =
            format!("ATTACH '{}' AS \"{}\" (READ_ONLY)", escaped_path, table);
        conn.execute(&attach_sql, []).map_err(|e| {
            tracing::error!(
                error = %e,
                table = %table,
                path = %path.display(),
                "ATTACH 表文件失败"
            );
            AppError::Internal(format!(
                "ATTACH 表 '{}' 失败: {}",
                table,
                crate::error::localizer::translate_raw(&e.to_string()),
            ))
        })?;
    }

    if !to_attach.is_empty() {
        let search_path = to_attach
            .iter()
            .map(|(t, _)| format!("{}.main", t))
            .collect::<Vec<_>>()
            .join(",");
        conn.execute(&format!("SET search_path = '{}'", search_path), [])
            .map_err(|e| {
                AppError::Internal(format!(
                    "设置 search_path 失败: {}",
                    crate::error::localizer::translate_raw(&e.to_string()),
                ))
            })?;
    }

    tracing::debug!(
        source_code = %source_code,
        attached = ?to_attach.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>(),
        "DuckDB 校验连接已就绪"
    );

    Ok(conn)
}

fn preprocess_sql_for_explain(sql: &str) -> String {
    let mut result = sql.to_string();

    // {{snippet: name}} → 空
    if let Ok(re) = Regex::new(r"\{\{snippet:\s*\w+\s*\}\}") {
        result = re.replace_all(&result, "").to_string();
    }

    // {{param}} → NULL (避免类型不匹配错误，NULL可与任意类型比较)
    if let Ok(re) = Regex::new(r"\{\{(\w+)\}\}") {
        result = re.replace_all(&result, "NULL").to_string();
    }

    // [[...]] → ...（(?s)：可选块可能跨多行,必须整块匹配剥掉标记）
    if let Ok(re) = Regex::new(r"(?s)\[\[(.*?)\]\]") {
        result = re.replace_all(&result, "$1").to_string();
    }

    result
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use super::*;

    fn table(name: &str) -> DataSourceTableSchemaVo {
        DataSourceTableSchemaVo {
            table_meta_id: 1,
            table_name: name.to_string(),
            comment_label: None,
            table_desc: None,
            columns: Vec::new(),
        }
    }

    #[test]
    fn keeps_only_exported_tables_case_insensitive() {
        let mut available: TableMap = HashMap::new();
        available.insert(
            "orders".to_string(),
            ("Orders".to_string(), PathBuf::from("/x/Orders.duckdb")),
        );
        let tables = vec![table("Orders"), table("customers")];
        let out =
            SqlQueryService::filter_tables_to_exported(tables, &available);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].table_name, "Orders");
    }

    #[test]
    fn empty_available_yields_empty() {
        let available: TableMap = HashMap::new();
        let out = SqlQueryService::filter_tables_to_exported(
            vec![table("a"), table("b")],
            &available,
        );
        assert!(out.is_empty());
    }

    #[test]
    fn apply_paging_wraps_with_limit_and_count() {
        let (paged, count) =
            SqlQueryService::apply_paging("SELECT 1 AS x", 2, 10);
        assert_eq!(
            paged,
            "SELECT * FROM (SELECT 1 AS x) AS _paged_query LIMIT 10 OFFSET 10"
        );
        assert!(count.unwrap().starts_with("SELECT COUNT(*)"));
    }

    #[test]
    fn apply_paging_non_positive_size_skips_paging() {
        // 公开 API 未传分页参数时（size=0）不做 LIMIT 包装，返回全量
        let (paged, count) =
            SqlQueryService::apply_paging("SELECT 1 AS x", 1, 0);
        assert_eq!(paged, "SELECT 1 AS x");
        assert_eq!(count, None);
        let (paged, count) =
            SqlQueryService::apply_paging("SELECT 1 AS x", 1, -3);
        assert_eq!(paged, "SELECT 1 AS x");
        assert_eq!(count, None);
    }

    #[test]
    fn explain_preprocess_strips_singleline_optional_block() {
        let out = preprocess_sql_for_explain(
            "SELECT * FROM t WHERE a = 1 [[AND b = {{b}}]]",
        );
        assert_eq!(out, "SELECT * FROM t WHERE a = 1 AND b = NULL");
    }

    #[test]
    fn explain_preprocess_strips_multiline_optional_block() {
        // 前端格式化器或用户手写都可能产生跨多行的可选块
        let out = preprocess_sql_for_explain(
            "SELECT * FROM t3 [[AND t3.code IN (\n  SELECT org_code\n  FROM sys_depart\n  WHERE zone = {{zone}}\n)]]",
        );
        assert!(!out.contains("[["), "got: {out}");
        assert!(!out.contains("]]"), "got: {out}");
        assert!(out.contains("zone = NULL"), "got: {out}");
        assert!(out.contains("t3.code IN ("), "got: {out}");
    }

    #[test]
    fn explain_preprocess_removes_snippet_refs() {
        let out = preprocess_sql_for_explain("SELECT {{snippet: foo}} FROM t");
        assert_eq!(out, "SELECT  FROM t");
    }
}
