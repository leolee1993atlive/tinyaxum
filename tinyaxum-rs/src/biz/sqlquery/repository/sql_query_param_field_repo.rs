use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder,
};

use crate::biz::sqlquery::model::{
    sql_query_param_field::SqlQueryParamField, sql_query_param_field_entity,
};

#[derive(Clone)]
pub struct SqlQueryParamFieldRepository {
    db: DatabaseConnection,
}

impl SqlQueryParamFieldRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn insert(
        &self,
        row: &SqlQueryParamField,
    ) -> Result<(), sea_orm::DbErr> {
        let am: sql_query_param_field_entity::ActiveModel = row.into();
        am.insert(&self.db).await?;
        Ok(())
    }

    pub async fn select_by_sql_query_id(
        &self,
        sql_query_id: &i64,
    ) -> Result<Vec<SqlQueryParamField>, sea_orm::DbErr> {
        let models = sql_query_param_field_entity::Entity::find()
            .filter(
                sql_query_param_field_entity::Column::SqlQueryId
                    .eq(*sql_query_id),
            )
            .filter(sql_query_param_field_entity::Column::DeletedBy.is_null())
            .filter(sql_query_param_field_entity::Column::DeletedDate.is_null())
            .order_by_asc(sql_query_param_field_entity::Column::FieldName)
            .all(&self.db)
            .await?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    pub async fn update_by_id(
        &self,
        row: &SqlQueryParamField,
        _param_field_id: &i64,
    ) -> Result<(), sea_orm::DbErr> {
        let am: sql_query_param_field_entity::ActiveModel = row.into();
        let _ = sql_query_param_field_entity::Entity::update(am)
            .validate()?
            .filter(sql_query_param_field_entity::Column::DeletedBy.is_null())
            .filter(sql_query_param_field_entity::Column::DeletedDate.is_null())
            .exec(&self.db)
            .await?;
        Ok(())
    }
}
