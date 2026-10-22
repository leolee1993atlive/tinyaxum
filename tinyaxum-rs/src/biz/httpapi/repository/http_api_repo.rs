use http_api_entity::ActiveModel;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, sea_query::Expr,
};
use serde::{Deserialize, Serialize};

use crate::biz::httpapi::model::{http_api::HttpApi, http_api_entity};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HttpApiPage {
    pub records: Vec<HttpApi>,
    pub total: u64,
    pub pages: u64,
}

#[derive(Clone)]
pub struct HttpApiRepository {
    db: DatabaseConnection,
}

impl HttpApiRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn select_by_id(
        &self,
        id: &i64,
    ) -> Result<Option<HttpApi>, sea_orm::DbErr> {
        use http_api_entity::{Column, Entity};
        let model = Entity::find_by_id(*id)
            .filter(Column::DeletedBy.is_null())
            .filter(Column::DeletedDate.is_null())
            .one(&self.db)
            .await?;
        Ok(model.map(|m| m.into()))
    }

    pub async fn select_by_path(
        &self,
        path: &str,
    ) -> Result<Option<HttpApi>, sea_orm::DbErr> {
        use http_api_entity::{Column, Entity};
        let model = Entity::find()
            .filter(Column::HttpApiPath.eq(path))
            .filter(Column::DeletedBy.is_null())
            .filter(Column::DeletedDate.is_null())
            .one(&self.db)
            .await?;
        Ok(model.map(|m| m.into()))
    }

    /// Replaces the legacy rbatis `htmlsql_select_page!` macro.
    /// Dynamic filter logic from `src/xml/http_api_repo.html`:
    ///   - keyword → http_api_name OR http_api_path OR http_api_desc LIKE
    pub async fn select_page(
        &self,
        keyword: &str,
        current: &i64,
        size: &i64,
        enabled: &Option<i32>,
        module: &Option<String>,
        category: &Option<String>,
    ) -> Result<HttpApiPage, sea_orm::DbErr> {
        let keyword = keyword.trim();
        let page_size = *size as u64;
        let page_no = *current as u64;

        use http_api_entity::{Column, Entity};
        let mut select = Entity::find()
            .filter(Column::DeletedBy.is_null())
            .filter(Column::DeletedDate.is_null());

        if !keyword.is_empty() {
            select = select.filter(
                Condition::any()
                    .add(Column::HttpApiName.contains(keyword))
                    .add(Column::HttpApiPath.contains(keyword))
                    .add(Column::HttpApiDesc.contains(keyword)),
            );
        }

        if let &Some(v) = enabled {
            select = select.filter(Column::Enabled.eq(v));
        }

        if let Some(m) = module
            && !m.trim().is_empty()
        {
            select = select.filter(Column::Module.eq(m.trim()));
        }
        if let Some(c) = category
            && !c.trim().is_empty()
        {
            select = select.filter(Column::Category.eq(c.trim()));
        }

        select = select.order_by_desc(Column::CreatedDate);

        let paginator = select.paginate(&self.db, page_size);
        let total = paginator.num_items().await?;
        let models = paginator.fetch_page(page_no.saturating_sub(1)).await?;

        let pages = if page_size == 0 {
            0
        } else {
            total.div_ceil(page_size)
        };

        Ok(HttpApiPage {
            records: models.into_iter().map(|m| m.into()).collect(),
            total,
            pages,
        })
    }

    pub async fn insert(&self, entity: &HttpApi) -> Result<(), sea_orm::DbErr> {
        let am: ActiveModel = entity.into();
        am.insert(&self.db).await?;
        Ok(())
    }

    pub async fn update_by_id(
        &self,
        entity: &HttpApi,
    ) -> Result<(), sea_orm::DbErr> {
        use http_api_entity::{Column, Entity};
        let am: ActiveModel = entity.into();
        let id = entity.http_api_id.unwrap_or(0);
        let _ = Entity::find_by_id(id)
            .filter(Column::DeletedBy.is_null())
            .filter(Column::DeletedDate.is_null())
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound(
                "Record not found".to_string(),
            ))?;
        let _ = am.update(&self.db).await?;
        Ok(())
    }

    pub async fn soft_delete_by_id(
        &self,
        id: &i64,
        user_id: &i64,
    ) -> Result<(), sea_orm::DbErr> {
        use chrono::Utc;
        use http_api_entity::{Column, Entity};
        Entity::update_many()
            .col_expr(Column::DeletedBy, Expr::value(*user_id))
            .col_expr(Column::DeletedDate, Expr::value(Utc::now()))
            .filter(Column::HttpApiId.eq(*id))
            .filter(Column::DeletedBy.is_null())
            .exec(&self.db)
            .await?;
        Ok(())
    }
}
