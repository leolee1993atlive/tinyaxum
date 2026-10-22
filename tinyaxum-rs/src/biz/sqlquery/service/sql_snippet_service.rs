use crate::{
    app::Infrastructure,
    biz::sqlquery::{
        handler::sql_snippet_handler::SqlSnippetPageQuery,
        model::sql_snippet::{
            SqlSnippet, SqlSnippetCreateBo, SqlSnippetDetailVo,
            SqlSnippetListVo, SqlSnippetUpdateBo,
        },
        repository::sql_snippet_repo::SqlSnippetRepository,
    },
    common::vo::page::{PageResult, normalize_paging, sanitize_sort},
    error::error::{AppError, AppResult},
    middleware::extractors::CurrentUser,
};

#[derive(Clone)]
pub struct SqlSnippetService {
    sql_snippet_repo: SqlSnippetRepository,
}

impl SqlSnippetService {
    pub fn new(infra: &Infrastructure) -> Self {
        SqlSnippetService {
            sql_snippet_repo: SqlSnippetRepository::new(infra.db.clone()),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn list(&self) -> AppResult<Vec<SqlSnippetListVo>> {
        let result = self.sql_snippet_repo.select_all().await?;
        tracing::info!(count = result.len(), "查询 SQL 片段列表成功");
        Ok(result
            .into_iter()
            .map(|entity| entity.to_list_vo())
            .collect())
    }

    #[tracing::instrument(skip(self, query))]
    pub async fn page(
        &self,
        query: SqlSnippetPageQuery,
    ) -> AppResult<PageResult<SqlSnippetListVo>> {
        let (current, size, _offset) = normalize_paging(&query.page);
        tracing::info!(
            current = current,
            size = size,
            keyword = ?query.keyword,
            "SQL 片段分页查询"
        );
        let allowed = [
            ("sqlSnippetId", "sql_snippet_id"),
            ("sqlSnippetName", "sql_snippet_name"),
            ("createdDate", "created_date"),
        ];
        let mut order_by =
            sanitize_sort(&query.page.sort, &query.page.order, &allowed);
        if order_by.is_none() {
            order_by = Some("sql_snippet_id desc".to_string());
        }

        let order_str = order_by.as_deref().unwrap_or("sql_snippet_id desc");
        let page = self
            .sql_snippet_repo
            .select_page(&query, current as u64, size as u64, order_str)
            .await
            .inspect_err(|e| {
                tracing::error!(error = %e, "SQL 片段分页数据获取失败");
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
            "SQL 片段分页查询完成"
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
    pub async fn get_sql_snippet_by_id(
        &self,
        id: i64,
    ) -> AppResult<SqlSnippetDetailVo> {
        let entity =
            self.sql_snippet_repo.select_by_id(&id).await?.ok_or_else(
                || AppError::NotFound(format!("SQL 片段不存在 (id={})", id)),
            )?;
        tracing::info!(id, "根据ID查询 SQL 片段成功");
        Ok(entity.to_detail_vo())
    }

    #[tracing::instrument(skip(self, bo, current_user), fields(username = %current_user.username))]
    pub async fn insert_by_bo(
        &self,
        bo: SqlSnippetCreateBo,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let entity = SqlSnippet::from_create_bo(bo, current_user);
        self.sql_snippet_repo.insert(&entity).await?;
        tracing::info!(
            sql_snippet_name = %entity.sql_snippet_name,
            "创建 SQL 片段成功"
        );
        Ok(())
    }

    #[tracing::instrument(skip(self, bo, current_user), fields(username = %current_user.username))]
    pub async fn update_by_bo(
        &self,
        bo: SqlSnippetUpdateBo,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let sql_snippet_id = bo
            .sql_snippet_id
            .ok_or_else(|| AppError::BadRequest("片段ID为空".to_string()))?;
        let mut entity = self
            .sql_snippet_repo
            .select_by_id(&sql_snippet_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "SQL 片段不存在 (id={})",
                    sql_snippet_id
                ))
            })?;

        entity.from_update_bo(bo, current_user);
        self.sql_snippet_repo
            .update_by_id(&entity, &sql_snippet_id)
            .await?;
        tracing::info!(sql_snippet_id, "更新 SQL 片段成功");
        Ok(())
    }

    #[tracing::instrument(skip(self, current_user), fields(id = id, username = %current_user.username))]
    pub async fn delete_by_id(
        &self,
        id: i64,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let mut entity =
            self.sql_snippet_repo.select_by_id(&id).await?.ok_or_else(
                || AppError::NotFound(format!("SQL 片段不存在 (id={})", id)),
            )?;

        entity.base_entity.delete(current_user.get_user_id());

        self.sql_snippet_repo.update_by_id(&entity, &id).await?;
        tracing::info!(id, "软删除 SQL 片段成功");
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(id = id))]
    pub async fn true_delete_by_id(&self, id: i64) -> AppResult<()> {
        self.sql_snippet_repo.delete_by_id(&id).await?;
        tracing::info!(id, "彻底删除 SQL 片段成功");
        Ok(())
    }
}
