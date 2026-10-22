use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder,
};

use crate::biz::sqlquery::model::{
    sql_query_result_field::SqlQueryResultField, sql_query_result_field_entity,
};

#[derive(Clone)]
pub struct SqlQueryResultFieldRepository {
    db: DatabaseConnection,
}

impl SqlQueryResultFieldRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn insert(
        &self,
        row: &SqlQueryResultField,
    ) -> Result<(), sea_orm::DbErr> {
        let am: sql_query_result_field_entity::ActiveModel = row.into();
        am.insert(&self.db).await?;
        Ok(())
    }

    pub async fn select_by_sql_query_id(
        &self,
        sql_query_id: &i64,
    ) -> Result<Vec<SqlQueryResultField>, sea_orm::DbErr> {
        let models = sql_query_result_field_entity::Entity::find()
            .filter(
                sql_query_result_field_entity::Column::SqlQueryId
                    .eq(*sql_query_id),
            )
            .filter(sql_query_result_field_entity::Column::DeletedBy.is_null())
            .filter(
                sql_query_result_field_entity::Column::DeletedDate.is_null(),
            )
            .order_by_asc(sql_query_result_field_entity::Column::FieldName)
            .all(&self.db)
            .await?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }
}
