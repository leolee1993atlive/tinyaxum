use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, Set,
};

use crate::{
    biz::dashboard::model::dashboard_config_entity, error::error::AppResult,
};

#[derive(Clone)]
pub struct DashboardConfigRepository {
    db: DatabaseConnection,
}

impl DashboardConfigRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn select_by_user_id(
        &self,
        user_id: &i64,
    ) -> AppResult<Option<dashboard_config_entity::Model>> {
        use dashboard_config_entity::{Column, Entity};
        Entity::find()
            .filter(Column::UserId.eq(*user_id))
            .filter(Column::DeletedBy.is_null())
            .filter(Column::DeletedDate.is_null())
            .one(&self.db)
            .await
            .map_err(Into::into)
    }

    pub async fn insert(
        &self,
        row: &dashboard_config_entity::Model,
    ) -> AppResult<()> {
        let active: dashboard_config_entity::ActiveModel =
            dashboard_config_entity::ActiveModel {
                user_id: Set(row.user_id),
                config_json: Set(row.config_json.clone()),
                version: Set(row.version),
                created_by: Set(row.created_by),
                created_date: Set(row.created_date),
                last_modified_by: Set(row.last_modified_by),
                last_modified_date: Set(row.last_modified_date),
                deleted_by: Set(row.deleted_by),
                deleted_date: Set(row.deleted_date),
            };
        active.insert(&self.db).await?;
        Ok(())
    }

    pub async fn update_config_by_user_id(
        &self,
        user_id: &i64,
        row: &dashboard_config_entity::Model,
    ) -> AppResult<()> {
        use dashboard_config_entity::{Column, Entity};
        let _ = Entity::find_by_id(*user_id)
            .filter(Column::DeletedBy.is_null())
            .filter(Column::DeletedDate.is_null())
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound(
                "Dashboard config not found".to_string(),
            ))?;
        let active: dashboard_config_entity::ActiveModel =
            dashboard_config_entity::ActiveModel {
                user_id: Set(row.user_id),
                config_json: Set(row.config_json.clone()),
                version: Set(row.version),
                created_by: Set(row.created_by),
                created_date: Set(row.created_date),
                last_modified_by: Set(row.last_modified_by),
                last_modified_date: Set(row.last_modified_date),
                deleted_by: Set(row.deleted_by),
                deleted_date: Set(row.deleted_date),
            };
        active.update(&self.db).await?;
        Ok(())
    }
}
