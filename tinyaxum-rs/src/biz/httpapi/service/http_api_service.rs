use std::collections::HashMap;

use serde_json::Value;

use crate::{
    app::Infrastructure,
    biz::{
        httpapi::{
            model::{
                http_api::{
                    HttpApi, HttpApiCreateBo, HttpApiDetailVo, HttpApiListVo,
                    HttpApiUpdateBo,
                },
                http_api_query::{HttpApiQuery, HttpApiQueryVo},
                http_api_query_entity,
            },
            repository::{
                http_api_query_repo::HttpApiQueryRepository,
                http_api_repo::HttpApiRepository,
            },
            service::compare_period,
        },
        sqlquery::service::sql_query_service::SqlQueryService,
    },
    common::{
        model::entity::{BaseEntity, BaseVo},
        vo::{page::PageResult, sql_run::QueryResultBundle},
    },
    error::error::{AppError, AppResult},
    util::template_renderer::TemplateRenderer,
};

/// 把 JSON 固定参数值转成 SQL 模板参数字符串（字符串原样，其余用 JSON 字面量）
fn json_value_to_param(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// 固定参数必须是 JSON 对象（键值都会以字符串形式参与 SQL 模板渲染）
fn validate_default_params(default_params: &Option<Value>) -> AppResult<()> {
    match default_params {
        None | Some(Value::Object(_)) => Ok(()),
        Some(other) => Err(AppError::BadRequest(format!(
            "defaultParams 必须是 JSON 对象, 收到: {}",
            other
        ))),
    }
}

#[derive(Clone)]
pub struct HttpApiService {
    http_api_repo: HttpApiRepository,
    http_api_query_repo: HttpApiQueryRepository,
    sql_query_service: SqlQueryService,
    template_renderer: TemplateRenderer,
}

impl HttpApiService {
    fn to_query_entity_models(
        queries: Vec<HttpApiQuery>,
    ) -> Vec<http_api_query_entity::Model> {
        queries
            .into_iter()
            .map(|q| http_api_query_entity::Model {
                http_api_query_id: q.http_api_query_id.unwrap_or(0),
                http_api_id: q.http_api_id,
                sql_query_id: q.sql_query_id,
                query_alias: q.query_alias,
                sort_order: q.sort_order,
                version: q.base_entity.version,
                created_by: q.base_entity.created_by,
                created_date: q.base_entity.created_date,
                last_modified_by: q.base_entity.last_modified_by,
                last_modified_date: q.base_entity.last_modified_date,
                deleted_by: q.base_entity.deleted_by,
                deleted_date: q.base_entity.deleted_date,
            })
            .collect()
    }

    pub fn new(
        infra: &Infrastructure,
        sql_query_service: SqlQueryService,
    ) -> Self {
        Self {
            http_api_repo: HttpApiRepository::new(infra.db.clone()),
            http_api_query_repo: HttpApiQueryRepository::new(infra.db.clone()),
            sql_query_service,
            template_renderer: TemplateRenderer::new(),
        }
    }

    #[tracing::instrument(skip(self), fields(id = id))]
    pub async fn get_by_id(&self, id: i64) -> AppResult<HttpApiDetailVo> {
        let api =
            self.http_api_repo.select_by_id(&id).await?.ok_or_else(|| {
                AppError::NotFound(format!("HTTP API不存在 (id={})", id))
            })?;
        let mut vo = api.to_detail_vo();
        let queries =
            self.http_api_query_repo.select_by_http_api_id(&id).await?;
        tracing::info!(query_count = queries.len(), "查询HTTP API详情完成");
        let mut query_vos = Vec::with_capacity(queries.len());
        for q in queries {
            let name = self
                .sql_query_service
                .get_sql_query_by_id(q.sql_query_id)
                .await
                .map(|detail| detail.sql_query_name)
                .unwrap_or_default();
            query_vos.push(HttpApiQueryVo {
                query_alias: q.query_alias,
                sql_query_id: q.sql_query_id,
                sql_query_name: name,
            });
        }
        vo.queries = query_vos;
        Ok(vo)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_page(
        &self,
        keyword: Option<String>,
        current: i64,
        size: i64,
        enabled: Option<i32>,
        module: Option<String>,
        category: Option<String>,
    ) -> AppResult<PageResult<HttpApiListVo>> {
        let keyword = keyword.unwrap_or_default();
        let current = if current < 1 { 1 } else { current };
        let size = if size < 1 { 10 } else { size };
        let page_data = self
            .http_api_repo
            .select_page(
                &keyword, &current, &size, &enabled, &module, &category,
            )
            .await?;
        let total = page_data.total as i64;
        let pages = if size == 0 {
            0
        } else {
            ((total + size - 1) / size).max(0)
        };
        let records: Vec<HttpApiListVo> = page_data
            .records
            .into_iter()
            .map(|api| HttpApiListVo {
                http_api_id: api.http_api_id.unwrap_or_default(),
                http_api_name: api.http_api_name.clone(),
                http_api_path: api.http_api_path.clone(),
                http_api_desc: api.http_api_desc.clone(),
                module: api.module.clone(),
                category: api.category.clone(),
                enabled: api.enabled,
                base_entity: BaseVo::new(api.base_entity.clone()),
            })
            .collect();
        let page = PageResult {
            records,
            total,
            pages,
            current,
            size,
        };
        tracing::info!(
            record_count = page.records.len(),
            total,
            pages,
            "分页查询HTTP API列表完成"
        );
        Ok(page)
    }

    #[tracing::instrument(skip(self, bo))]
    pub async fn create(
        &self,
        bo: HttpApiCreateBo,
        user_id: i64,
    ) -> AppResult<()> {
        tracing::info!(
            name = %bo.http_api_name,
            path = %bo.http_api_path,
            user_id,
            "开始创建HTTP API"
        );
        // 1. Check path uniqueness
        if self
            .http_api_repo
            .select_by_path(&bo.http_api_path)
            .await?
            .is_some()
        {
            tracing::warn!(
                path = %bo.http_api_path,
                "创建HTTP API失败, 路径已存在"
            );
            return Err(AppError::BadRequest("路径已存在".to_string()));
        }
        validate_default_params(&bo.default_params)?;
        // 2. Insert main entity
        let queries = bo.queries.clone();
        let api = HttpApi::from_create_bo(bo, user_id);
        self.http_api_repo.insert(&api).await?;
        // 3. Retrieve id by querying with the path
        let inserted = self
            .http_api_repo
            .select_by_path(&api.http_api_path)
            .await?
            .ok_or_else(|| {
                AppError::Internal("创建API后未能查询到记录".to_string())
            })?;
        let api_id = inserted.http_api_id.ok_or_else(|| {
            AppError::Internal("创建API后未获取到ID".to_string())
        })?;
        // 4. Insert query associations
        let query_entities: Vec<HttpApiQuery> = queries
            .into_iter()
            .map(|q| HttpApiQuery {
                http_api_query_id: None,
                http_api_id: api_id,
                sql_query_id: q.sql_query_id,
                query_alias: q.query_alias,
                sort_order: q.sort_order,
                base_entity: BaseEntity::new(user_id),
            })
            .collect();
        let query_count = query_entities.len();
        if !query_entities.is_empty() {
            let entity_models = Self::to_query_entity_models(query_entities);
            self.http_api_query_repo
                .insert_batch(&entity_models)
                .await?;
        } else {
            tracing::debug!(api_id, "HTTP API未关联SQL查询");
        }
        tracing::info!(
            api_id,
            query_count,
            path = %api.http_api_path,
            "HTTP API创建成功"
        );
        Ok(())
    }

    #[tracing::instrument(skip(self, bo))]
    pub async fn update(
        &self,
        bo: HttpApiUpdateBo,
        user_id: i64,
    ) -> AppResult<()> {
        let id = bo.http_api_id.unwrap_or(0);
        let queries = bo.queries.clone();
        tracing::info!(api_id = id, user_id, "开始更新HTTP API");
        validate_default_params(&bo.default_params)?;
        let mut entity =
            self.http_api_repo.select_by_id(&id).await?.ok_or_else(|| {
                AppError::NotFound(format!("API不存在 (id={})", id))
            })?;
        entity.from_update_bo(bo, user_id);
        self.http_api_repo.update_by_id(&entity).await?;
        // Update query associations: delete old, insert new
        self.http_api_query_repo.delete_by_http_api_id(&id).await?;
        let query_entities: Vec<HttpApiQuery> = queries
            .into_iter()
            .map(|q| HttpApiQuery {
                http_api_query_id: None,
                http_api_id: id,
                sql_query_id: q.sql_query_id,
                query_alias: q.query_alias,
                sort_order: q.sort_order,
                base_entity: BaseEntity::new(user_id),
            })
            .collect();
        let query_count = query_entities.len();
        if !query_entities.is_empty() {
            let entity_models = Self::to_query_entity_models(query_entities);
            self.http_api_query_repo
                .insert_batch(&entity_models)
                .await?;
        } else {
            tracing::debug!(api_id = id, "HTTP API更新后未关联SQL查询");
        }
        tracing::info!(api_id = id, query_count, "HTTP API更新成功");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete(&self, id: i64, user_id: i64) -> AppResult<()> {
        self.http_api_repo.soft_delete_by_id(&id, &user_id).await?;
        tracing::info!(api_id = id, "HTTP API删除成功");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn enable(&self, id: i64, user_id: i64) -> AppResult<()> {
        let mut entity =
            self.http_api_repo.select_by_id(&id).await?.ok_or_else(|| {
                AppError::NotFound(format!("API不存在 (id={})", id))
            })?;
        entity.enabled = 1;
        entity.base_entity.update(user_id);
        self.http_api_repo.update_by_id(&entity).await?;
        tracing::info!(api_id = id, "HTTP API启用成功");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn disable(&self, id: i64, user_id: i64) -> AppResult<()> {
        let mut entity =
            self.http_api_repo.select_by_id(&id).await?.ok_or_else(|| {
                AppError::NotFound(format!("API不存在 (id={})", id))
            })?;
        entity.enabled = 0;
        entity.base_entity.update(user_id);
        self.http_api_repo.update_by_id(&entity).await?;
        tracing::info!(api_id = id, "HTTP API禁用成功");
        Ok(())
    }

    #[tracing::instrument(skip(self, params))]
    pub async fn execute_public_api(
        &self,
        path: &str,
        params: HashMap<String, String>,
    ) -> AppResult<Value> {
        // axum 通配符捕获的 path 不含前导斜杠，配置里常带，这里归一化
        let path = path.trim_start_matches('/');
        tracing::info!(
            path,
            params = ?params,
            "收到公开API请求"
        );

        // 1. Find API by path：请求侧去掉前导斜杠后，兼容带/不带斜杠两种存量配置
        let api = match self.http_api_repo.select_by_path(path).await? {
            Some(api) => api,
            None => self
                .http_api_repo
                .select_by_path(&format!("/{}", path))
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!("API not found: {}", path))
                })?,
        };
        let api_id = api.http_api_id.unwrap_or_default();

        // 2. Check enabled
        if api.enabled != 1 {
            tracing::warn!(api_id, "公开API请求失败, API已禁用");
            return Err(AppError::Forbidden("API is disabled".to_string()));
        }
        tracing::debug!(
            api_id,
            name = %api.http_api_name,
            "查询到公开API"
        );

        // 0. 分页保留参数：pageNo/pageSize 都未传 → 不分页返回全量；
        //    任一传入 → 分页（缺省页码 1、页大小 1000）。其余参数透传 SQL。
        let (current, size) = if params.contains_key("pageNo")
            || params.contains_key("pageSize")
        {
            (
                params
                    .get("pageNo")
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(1)
                    .max(1),
                params
                    .get("pageSize")
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(1000)
                    .clamp(1, 10000),
            )
        } else {
            (1, 0)
        };
        let mut sql_params: HashMap<String, String> = params
            .into_iter()
            .filter(|(k, _)| k != "pageNo" && k != "pageSize")
            .collect();
        // API 固定参数覆盖同名请求参数（用于旧接口口径：天数窗口、ICD 列表、
        // 写死科室等——请求方误传时以配置为准）
        if let Some(Value::Object(defaults)) = api.default_params.as_ref() {
            for (k, v) in defaults {
                if !v.is_null() {
                    sql_params.insert(k.clone(), json_value_to_param(v));
                }
            }
        }

        // 3. Get associated queries
        let queries = self
            .http_api_query_repo
            .select_by_http_api_id(&api_id)
            .await?;
        tracing::debug!(
            api_id,
            query_count = queries.len(),
            "公开API关联SQL查询"
        );
        // 4. Execute SQL for all queries (with optional YoY/MoM for non-collection queries)
        let mut bundles: HashMap<&str, QueryResultBundle> = HashMap::new();
        for q in &queries {
            tracing::debug!(
                api_id,
                sql_query_id = q.sql_query_id,
                alias = %q.query_alias,
                "执行关联SQL查询"
            );

            // 4a. Current period
            let current_res = self
                .sql_query_service
                .run_sql_query(
                    q.sql_query_id,
                    Some(sql_params.clone()),
                    0,
                    current,
                    size,
                )
                .await
                .map_err(|e| {
                    tracing::error!(
                        api_id,
                        sql_query_id = q.sql_query_id,
                        error = %e,
                        "SQL查询执行失败"
                    );
                    AppError::Internal(format!("SQL execution failed: {}", e))
                })?;

            // 4b. Determine whether to fetch YoY/MoM
            let date_pair = sql_params
                .get("startDate")
                .and_then(|s| compare_period::parse_date_param(s))
                .zip(
                    sql_params
                        .get("endDate")
                        .and_then(|s| compare_period::parse_date_param(s)),
                );
            let (yoy_opt, mom_opt) =
                if current_res.is_collection == 0 {
                    if let Some((start, end)) = date_pair {
                        let yoy_params = compare_period::yoy_range(start, end)
                            .map(|(s, e)| {
                                let mut p = sql_params.clone();
                                p.insert(
                                    "startDate".into(),
                                    s.format("%Y-%m-%d").to_string(),
                                );
                                p.insert(
                                    "endDate".into(),
                                    e.format("%Y-%m-%d").to_string(),
                                );
                                p
                            });
                        let mom_params = compare_period::mom_range(start, end)
                            .map(|(s, e)| {
                                let mut p = sql_params.clone();
                                p.insert(
                                    "startDate".into(),
                                    s.format("%Y-%m-%d").to_string(),
                                );
                                p.insert(
                                    "endDate".into(),
                                    e.format("%Y-%m-%d").to_string(),
                                );
                                p
                            });

                        let sql_id = q.sql_query_id;
                        let alias = q.query_alias.clone();
                        let svc = &self.sql_query_service;

                        let yoy_fut = async {
                            match yoy_params {
                        Some(p) => svc
                            .run_sql_query(sql_id, Some(p), 0, current, size)
                            .await
                            .map_err(|e| {
                                tracing::warn!(
                                    api_id,
                                    alias = %alias,
                                    error = %e,
                                    "同比SQL执行失败,_yoy 将置为占位空对象"
                                );
                                e
                            })
                            .ok(),
                        None => None,
                    }
                        };
                        let mom_fut = async {
                            match mom_params {
                        Some(p) => svc
                            .run_sql_query(sql_id, Some(p), 0, current, size)
                            .await
                            .map_err(|e| {
                                tracing::warn!(
                                    api_id,
                                    alias = %q.query_alias,
                                    error = %e,
                                    "环比SQL执行失败,_mom 将置为占位空对象"
                                );
                                e
                            })
                            .ok(),
                        None => None,
                    }
                        };

                        tokio::join!(yoy_fut, mom_fut)
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };

            bundles.insert(
                q.query_alias.as_str(),
                QueryResultBundle {
                    current: current_res,
                    yoy: yoy_opt,
                    mom: mom_opt,
                },
            );
        }
        // 5. Build template data
        let template_data = self.template_renderer.build_template_data(bundles);
        tracing::debug!(
            api_id,
            data = serde_json::to_string(&template_data).unwrap_or_default(),
            "模板数据构建完成"
        );
        // 6. Render response template if configured, otherwise return data directly
        let v = if let Some(template) = api.response_template.as_ref() {
            tracing::debug!(api_id, "开始渲染自定义模板");
            let rendered = self
                .template_renderer
                .render_template(template.as_str(), &template_data)
                .unwrap_or_else(|e| {
                    tracing::error!(
                        api_id,
                        error = %e,
                        "模板渲染失败"
                    );
                    serde_json::to_string(&serde_json::json!(
                        {"error": format!("template_error: {}", e)}
                    ))
                    .unwrap_or_else(|e| {
                        tracing::error!("JSON serialization failed: {}", e);
                        "{}".to_string()
                    })
                });
            serde_json::from_str(&rendered).unwrap_or(Value::Null)
        } else {
            tracing::debug!(api_id, "无响应模板, 直接返回查询数据");
            template_data
        };
        tracing::info!(api_id, "公开API请求处理完成");
        Ok(v)
    }
}
