use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, Order,
    PaginatorTrait, QueryFilter, QueryOrder,
};
use serde_json::Value;

use crate::{
    biz::sqlquery::model::{
        sql_query::{SqlQuery, SqlQueryListRow, SqlQueryPageQuery},
        sql_query_entity::{ActiveModel, Column, Entity as BizSqlQuery, Model},
    },
    common::{model::entity::BaseEntity, repo::str_trim_or_empty},
};

fn derive_viz_kind(
    is_collection: i8,
    viz_config: Option<&Value>,
) -> Option<String> {
    if is_collection == 0 {
        return Some("scalar".to_string());
    }
    let kind = viz_config
        .and_then(|v| v.get("kind"))
        .and_then(|k| k.as_str())
        .unwrap_or("table");
    let normalized = if kind == "chart" { "chart" } else { "table" };
    Some(normalized.to_string())
}

impl From<&Model> for SqlQueryListRow {
    fn from(m: &Model) -> Self {
        let viz_kind = derive_viz_kind(m.is_collection, m.viz_config.as_ref());
        SqlQueryListRow {
            sql_query_id: Some(m.sql_query_id),
            sql_query_name: m.sql_query_name.clone(),
            sql_query_desc: m.sql_query_desc.clone(),
            data_source_id: m.data_source_id,
            is_collection: m.is_collection,
            viz_kind,
            module: m.module.clone(),
            category: m.category.clone(),
            base_entity: BaseEntity {
                version: m.version,
                created_by: m.created_by,
                created_date: m.created_date,
                last_modified_by: m.last_modified_by,
                last_modified_date: m.last_modified_date,
                deleted_by: m.deleted_by,
                deleted_date: m.deleted_date,
            },
        }
    }
}

impl From<&Model> for SqlQuery {
    fn from(m: &Model) -> Self {
        SqlQuery {
            sql_query_id: Some(m.sql_query_id),
            sql_query_name: m.sql_query_name.clone(),
            sql_query_desc: m.sql_query_desc.clone(),
            sql_query_content: m.sql_query_content.clone(),
            data_source_id: m.data_source_id,
            result_meta_updated_at: m.result_meta_updated_at,
            is_collection: m.is_collection,
            viz_config: m.viz_config.clone(),
            module: m.module.clone(),
            category: m.category.clone(),
            base_entity: BaseEntity {
                version: m.version,
                created_by: m.created_by,
                created_date: m.created_date,
                last_modified_by: m.last_modified_by,
                last_modified_date: m.last_modified_date,
                deleted_by: m.deleted_by,
                deleted_date: m.deleted_date,
            },
        }
    }
}

fn to_active(sq: &SqlQuery) -> ActiveModel {
    ActiveModel {
        sql_query_id: match sq.sql_query_id {
            Some(id) => Set(id),
            None => NotSet,
        },
        sql_query_name: Set(sq.sql_query_name.clone()),
        sql_query_desc: Set(sq.sql_query_desc.clone()),
        sql_query_content: Set(sq.sql_query_content.clone()),
        data_source_id: Set(sq.data_source_id),
        result_meta_updated_at: Set(sq.result_meta_updated_at),
        is_collection: Set(sq.is_collection),
        viz_config: Set(sq.viz_config.clone()),
        module: Set(sq.module.clone()),
        category: Set(sq.category.clone()),
        version: Set(sq.base_entity.version),
        created_by: Set(sq.base_entity.created_by),
        created_date: Set(sq.base_entity.created_date),
        last_modified_by: Set(sq.base_entity.last_modified_by),
        last_modified_date: Set(sq.base_entity.last_modified_date),
        deleted_by: Set(sq.base_entity.deleted_by),
        deleted_date: Set(sq.base_entity.deleted_date),
    }
}

fn active_filter() -> Condition {
    Condition::all()
        .add(Column::DeletedBy.is_null())
        .add(Column::DeletedDate.is_null())
}

/// Falls back to `sql_query_id desc`.
fn parse_order(order_by: &str) -> (Column, Order) {
    let mut parts = order_by.split_whitespace();
    let col = match parts.next().unwrap_or("sql_query_id") {
        "sql_query_id" => Column::SqlQueryId,
        "sql_query_name" => Column::SqlQueryName,
        "data_source_id" => Column::DataSourceId,
        "is_collection" => Column::IsCollection,
        "created_date" => Column::CreatedDate,
        _ => Column::SqlQueryId,
    };
    let desc = !parts.next().is_some_and(|d| d.eq_ignore_ascii_case("asc"));
    if desc {
        (col, Order::Desc)
    } else {
        (col, Order::Asc)
    }
}

pub struct SqlQueryPage {
    pub records: Vec<SqlQueryListRow>,
    pub total: u64,
}

#[derive(Clone)]
pub struct SqlQueryRepository {
    db: DatabaseConnection,
}

impl SqlQueryRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// 列表用：不读取 `sql_query_content` 大字段;但会读 `viz_config` 以提取 `viz_kind` 摘要。
    pub async fn select_all_lite(
        &self,
    ) -> Result<Vec<SqlQueryListRow>, sea_orm::DbErr> {
        let models = BizSqlQuery::find()
            .filter(active_filter())
            .order_by_desc(Column::SqlQueryId)
            .all(&self.db)
            .await?;
        Ok(models.iter().map(SqlQueryListRow::from).collect())
    }

    /// Replaces the rbatis `htmlsql_select_page!` macro with a SeaORM
    /// query builder that replicates the same dynamic-SQL logic from
    /// `src/xml/sql_query_repo.html`.
    ///
    /// Original conditions:
    ///   WHERE deleted_by IS NULL AND deleted_date IS NULL
    ///     AND data_source_id = ?     (if > 0)
    ///     AND sql_query_name LIKE ?  (if sql_query_name != '')
    ///     AND sql_query_name LIKE ?  (if keyword != '')
    ///   ORDER BY ${order_by}
    pub async fn select_page(
        &self,
        query: &SqlQueryPageQuery,
        page_no: u64,
        page_size: u64,
        order_by: &str,
    ) -> Result<SqlQueryPage, sea_orm::DbErr> {
        let sql_query_name = str_trim_or_empty(&query.sql_query_name);
        let keyword = str_trim_or_empty(&query.keyword);
        let data_source_id = query.data_source_id.unwrap_or(0);

        let mut select = BizSqlQuery::find().filter(active_filter());

        if data_source_id > 0 {
            select = select.filter(Column::DataSourceId.eq(data_source_id));
        }
        if !sql_query_name.is_empty() {
            select =
                select.filter(Column::SqlQueryName.contains(&sql_query_name));
        }
        if !keyword.is_empty() {
            select = select.filter(Column::SqlQueryName.contains(&keyword));
        }
        if let Some(ref m) = query.module
            && !m.trim().is_empty()
        {
            select = select.filter(Column::Module.eq(m.trim()));
        }
        if let Some(ref c) = query.category
            && !c.trim().is_empty()
        {
            select = select.filter(Column::Category.eq(c.trim()));
        }

        let (order_col, order_dir) = parse_order(order_by);
        select = select.order_by(order_col, order_dir);

        // Paginate: count + fetch
        let paginator = select.paginate(&self.db, page_size);
        let total = paginator.num_items().await?;
        let models = paginator.fetch_page(page_no.saturating_sub(1)).await?;

        Ok(SqlQueryPage {
            records: models.iter().map(SqlQueryListRow::from).collect(),
            total,
        })
    }

    pub async fn select_by_id(
        &self,
        sql_query_id: &i64,
    ) -> Result<Option<SqlQuery>, sea_orm::DbErr> {
        let model = BizSqlQuery::find_by_id(*sql_query_id)
            .filter(active_filter())
            .one(&self.db)
            .await?;
        Ok(model.as_ref().map(SqlQuery::from))
    }

    pub async fn select_latest_by_name_for_user(
        &self,
        sql_query_name: &str,
        created_by: &i64,
    ) -> Result<Option<SqlQuery>, sea_orm::DbErr> {
        let model = BizSqlQuery::find()
            .filter(Column::SqlQueryName.eq(sql_query_name))
            .filter(Column::CreatedBy.eq(*created_by))
            .filter(active_filter())
            .order_by_desc(Column::SqlQueryId)
            .one(&self.db)
            .await?;
        Ok(model.as_ref().map(SqlQuery::from))
    }

    pub async fn insert(
        &self,
        data_source: &SqlQuery,
    ) -> Result<i64, sea_orm::DbErr> {
        let active = to_active(data_source);
        let result = BizSqlQuery::insert(active).exec(&self.db).await?;
        Ok(result.last_insert_id)
    }

    pub async fn update_by_id(
        &self,
        data_source: &SqlQuery,
        _sql_query_id: &i64,
    ) -> Result<(), sea_orm::DbErr> {
        let active = to_active(data_source);
        let _ = BizSqlQuery::update(active)
            .validate()?
            .filter(active_filter())
            .exec(&self.db)
            .await?;
        Ok(())
    }

    pub async fn delete_by_id(
        &self,
        sql_query_id: &i64,
    ) -> Result<(), sea_orm::DbErr> {
        BizSqlQuery::delete_by_id(*sql_query_id)
            .exec(&self.db)
            .await?;
        Ok(())
    }
}
