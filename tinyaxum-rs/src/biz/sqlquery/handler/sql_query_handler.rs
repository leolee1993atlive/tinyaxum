use std::{collections::HashMap, sync::Arc};

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use validator::Validate;

use crate::{
    app::AppState,
    biz::{
        datasource::model::data_source_schema_vo::DataSourceSchemaVo,
        sqlquery::model::sql_query::{
            SqlQueryCreateBo, SqlQueryDetailVo, SqlQueryListVo,
            SqlQueryPageQuery, SqlQueryRunAdhocBo, SqlQueryUpdateBo,
            SqlQueryVizConfigSaveBo, validate_viz_config_json,
        },
    },
    common::vo::{
        page::{
            PageQuery, PageResult, deserialize_opt_i64_from_str,
            normalize_paging,
        },
        response::R,
        sql_run::SqlRunResultVo,
    },
    error::error::AppError,
    middleware::extractors::CurrentUser,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunSqlQueryParams {
    #[serde(default, deserialize_with = "deserialize_opt_i64_from_str")]
    pub page_no: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_opt_i64_from_str")]
    pub page_size: Option<i64>,
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

#[tracing::instrument(skip(state))]
pub async fn list_sql_query(
    State(state): State<Arc<AppState>>,
) -> Result<R<Vec<SqlQueryListVo>>, AppError> {
    let data = state.services.sql_query_service.list().await?;
    Ok(R::ok_with_data(data))
}

#[tracing::instrument(skip(state, query))]
pub async fn list_sql_query_page(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SqlQueryPageQuery>,
) -> Result<R<PageResult<SqlQueryListVo>>, AppError> {
    let result = state.services.sql_query_service.page(query).await?;
    Ok(R::ok_with_data(result))
}

#[tracing::instrument(skip(state), fields(id = id))]
pub async fn get_sql_query_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<SqlQueryDetailVo>, AppError> {
    let sql_query = state
        .services
        .sql_query_service
        .get_sql_query_by_id(id)
        .await?;
    Ok(R::ok_with_data(sql_query))
}

#[tracing::instrument(skip(state, bo), fields(username = %current_user.username))]
pub async fn create_sql_query(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<SqlQueryCreateBo>,
) -> Result<R<i64>, AppError> {
    bo.validate()?;
    validate_viz_config_json(bo.viz_config.as_ref())?;
    let id = state
        .services
        .sql_query_service
        .insert_by_bo(bo, &current_user)
        .await?;
    Ok(R::ok_with_data(id))
}

#[tracing::instrument(skip(state, bo), fields(id = id, username = %current_user.username))]
pub async fn save_sql_query_viz_config(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(bo): Json<SqlQueryVizConfigSaveBo>,
) -> Result<R<()>, AppError> {
    validate_viz_config_json(bo.viz_config.as_ref())?;
    state
        .services
        .sql_query_service
        .update_viz_config(id, bo.viz_config, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state, bo), fields(username = %current_user.username))]
pub async fn update_sql_query(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(bo): Json<SqlQueryUpdateBo>,
) -> Result<R<()>, AppError> {
    bo.validate()?;
    validate_viz_config_json(bo.viz_config.as_ref())?;
    state
        .services
        .sql_query_service
        .update_by_bo(bo, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state, current_user), fields(id = id, username = %current_user.username))]
pub async fn delete_sql_query(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .sql_query_service
        .delete_by_id(id, &current_user)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state), fields(id = id))]
pub async fn true_delete_sql_query(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<R<()>, AppError> {
    state
        .services
        .sql_query_service
        .true_delete_by_id(id)
        .await?;
    Ok(R::ok())
}

#[tracing::instrument(skip(state, current_user), fields(id = id, username = %current_user.username))]
pub async fn run_sql_query(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Query(query): Query<RunSqlQueryParams>,
) -> Result<R<SqlRunResultVo>, AppError> {
    let page_query = PageQuery {
        page_no: query.page_no,
        page_size: query.page_size,
        ..Default::default()
    };
    let (current, size, _) = normalize_paging(&page_query);
    let params = if query.extra.is_empty() {
        None
    } else {
        Some(query.extra)
    };
    let out = state
        .services
        .sql_query_service
        .run_sql_query(id, params, current_user.get_user_id(), current, size)
        .await?;
    Ok(R::ok_with_data(out))
}

#[tracing::instrument(skip(state, bo))]
pub async fn run_adhoc_sql_query(
    State(state): State<Arc<AppState>>,
    Json(bo): Json<SqlQueryRunAdhocBo>,
) -> Result<R<SqlRunResultVo>, AppError> {
    bo.validate()?;
    let params = bo.params.filter(|p| !p.is_empty());
    let ds_id = bo.data_source_id.unwrap_or(0);
    let page_query = PageQuery {
        page_no: bo.page_no,
        page_size: bo.page_size,
        ..Default::default()
    };
    let (current, size, _) = normalize_paging(&page_query);
    let rows = state
        .services
        .sql_query_service
        .run_adhoc_sql(bo.sql_query_content, params, ds_id, current, size)
        .await?;
    Ok(R::ok_with_data(rows))
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlValidateRequest {
    pub sql_query_content: String,
    pub data_source_id: i64,
}

#[tracing::instrument(skip(state, bo))]
pub async fn validate_sql_query(
    State(state): State<Arc<AppState>>,
    Json(bo): Json<SqlValidateRequest>,
) -> Result<R<Option<String>>, AppError> {
    let result = state
        .services
        .sql_query_service
        .validate_sql(bo.sql_query_content, bo.data_source_id)
        .await?;
    Ok(R::ok_with_data(result))
}

#[tracing::instrument(skip(state), fields(data_source_id = data_source_id))]
pub async fn get_query_schema(
    State(state): State<Arc<AppState>>,
    Path(data_source_id): Path<i64>,
) -> Result<R<DataSourceSchemaVo>, AppError> {
    let schema = state
        .services
        .sql_query_service
        .get_query_schema(data_source_id)
        .await?;
    Ok(R::ok_with_data(schema))
}
