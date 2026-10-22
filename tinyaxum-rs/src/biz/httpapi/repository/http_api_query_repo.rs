use sea_orm::{
    ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};

use crate::{
    biz::httpapi::model::http_api_query_entity, error::error::AppResult,
};

#[derive(Clone)]
pub struct HttpApiQueryRepository {
    db: DatabaseConnection,
}

impl HttpApiQueryRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn select_by_http_api_id(
        &self,
        http_api_id: &i64,
    ) -> AppResult<Vec<http_api_query_entity::Model>> {
        use http_api_query_entity::{Column, Entity};
        Entity::find()
            .filter(Column::HttpApiId.eq(*http_api_id))
            .filter(Column::DeletedBy.is_null())
            .filter(Column::DeletedDate.is_null())
            .order_by_asc(Column::SortOrder)
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    pub async fn insert_batch(
        &self,
        entities: &[http_api_query_entity::Model],
    ) -> AppResult<()> {
        if entities.is_empty() {
            return Ok(());
        }
        let active_models: Vec<http_api_query_entity::ActiveModel> = entities
            .iter()
            .map(|m| http_api_query_entity::ActiveModel {
                http_api_query_id: ActiveValue::NotSet,
                http_api_id: ActiveValue::Set(m.http_api_id),
                sql_query_id: ActiveValue::Set(m.sql_query_id),
                query_alias: ActiveValue::Set(m.query_alias.clone()),
                sort_order: ActiveValue::Set(m.sort_order),
                version: ActiveValue::Set(m.version),
                created_by: ActiveValue::Set(m.created_by),
                created_date: ActiveValue::Set(m.created_date),
                last_modified_by: ActiveValue::Set(m.last_modified_by),
                last_modified_date: ActiveValue::Set(m.last_modified_date),
                deleted_by: ActiveValue::Set(m.deleted_by),
                deleted_date: ActiveValue::Set(m.deleted_date),
            })
            .collect();
        http_api_query_entity::Entity::insert_many(active_models)
            .exec(&self.db)
            .await?;
        Ok(())
    }

    pub async fn delete_by_http_api_id(
        &self,
        http_api_id: &i64,
    ) -> AppResult<()> {
        use http_api_query_entity::{Column, Entity};
        Entity::delete_many()
            .filter(Column::HttpApiId.eq(*http_api_id))
            .exec(&self.db)
            .await?;
        Ok(())
    }
}
