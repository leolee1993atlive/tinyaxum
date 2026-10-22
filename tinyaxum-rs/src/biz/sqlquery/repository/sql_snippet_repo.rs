use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    Order, PaginatorTrait, QueryFilter, QueryOrder,
};

use crate::{
    biz::sqlquery::{
        handler::sql_snippet_handler::SqlSnippetPageQuery,
        model::{
            sql_snippet::SqlSnippet,
            sql_snippet_entity::{
                ActiveModel, Column, Entity as SqlSnippetEntity,
            },
        },
    },
    common::repo::str_trim_or_empty,
};

fn active_filter() -> Condition {
    Condition::all()
        .add(Column::DeletedBy.is_null())
        .add(Column::DeletedDate.is_null())
}

pub struct SqlSnippetPage {
    pub records: Vec<SqlSnippet>,
    pub total: u64,
}

fn parse_order(order_by: &str) -> (Column, Order) {
    let mut parts = order_by.split_whitespace();
    let col = match parts.next().unwrap_or("sql_snippet_id") {
        "sql_snippet_id" => Column::SqlSnippetId,
        "sql_snippet_name" => Column::SqlSnippetName,
        "created_date" => Column::CreatedDate,
        _ => Column::SqlSnippetId,
    };
    let desc = !parts.next().is_some_and(|d| d.eq_ignore_ascii_case("asc"));
    if desc {
        (col, Order::Desc)
    } else {
        (col, Order::Asc)
    }
}

#[derive(Clone)]
pub struct SqlSnippetRepository {
    db: DatabaseConnection,
}

impl SqlSnippetRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn select_all(&self) -> Result<Vec<SqlSnippet>, sea_orm::DbErr> {
        let models = SqlSnippetEntity::find()
            .filter(active_filter())
            .all(&self.db)
            .await?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    pub async fn select_page(
        &self,
        query: &SqlSnippetPageQuery,
        page_no: u64,
        page_size: u64,
        order_by: &str,
    ) -> Result<SqlSnippetPage, sea_orm::DbErr> {
        let sql_snippet_name = str_trim_or_empty(&query.sql_snippet_name);
        let tag = str_trim_or_empty(&query.tag);
        let keyword = str_trim_or_empty(&query.keyword);

        let mut select = SqlSnippetEntity::find()
            .filter(active_filter())
            .filter(Column::SqlSnippetId.ne(0));

        if !sql_snippet_name.is_empty() {
            select = select
                .filter(Column::SqlSnippetName.contains(&sql_snippet_name));
        }
        if !tag.is_empty() {
            select = select.filter(Column::SqlSnippetTag.contains(&tag));
        }
        if !keyword.is_empty() {
            select = select.filter(
                Condition::any()
                    .add(Column::SqlSnippetName.contains(&keyword))
                    .add(Column::SqlSnippetDesc.contains(&keyword))
                    .add(Column::SqlSnippetTag.contains(&keyword)),
            );
        }

        let (order_col, order_dir) = parse_order(order_by);
        select = select.order_by(order_col, order_dir);

        let paginator = select.paginate(&self.db, page_size);
        let total = paginator.num_items().await?;
        let models = paginator.fetch_page(page_no.saturating_sub(1)).await?;

        Ok(SqlSnippetPage {
            records: models.into_iter().map(|m| m.into()).collect(),
            total,
        })
    }

    pub async fn select_by_id(
        &self,
        sql_snippet_id: &i64,
    ) -> Result<Option<SqlSnippet>, sea_orm::DbErr> {
        let model = SqlSnippetEntity::find_by_id(*sql_snippet_id)
            .filter(active_filter())
            .one(&self.db)
            .await?;
        Ok(model.map(|m| m.into()))
    }

    pub async fn insert(
        &self,
        data_source: &SqlSnippet,
    ) -> Result<(), sea_orm::DbErr> {
        let am: ActiveModel = data_source.into();
        am.insert(&self.db).await?;
        Ok(())
    }

    pub async fn update_by_id(
        &self,
        data_source: &SqlSnippet,
        _sql_snippet_id: &i64,
    ) -> Result<(), sea_orm::DbErr> {
        let am: ActiveModel = data_source.into();
        let _ = SqlSnippetEntity::update(am)
            .validate()?
            .filter(active_filter())
            .exec(&self.db)
            .await?;
        Ok(())
    }

    pub async fn delete_by_id(
        &self,
        sql_snippet_id: &i64,
    ) -> Result<(), sea_orm::DbErr> {
        SqlSnippetEntity::delete_by_id(*sql_snippet_id)
            .exec(&self.db)
            .await?;
        Ok(())
    }
}
