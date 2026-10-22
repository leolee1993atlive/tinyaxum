use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};

use crate::biz::datasource::model::{
    data_source_column_meta_entity::{
        Column as ColumnMetaColumn, Entity as ColumnMetaEntity,
    },
    data_source_table_meta::DataSourceTableMeta,
    data_source_table_meta_entity::{
        ActiveModel, Column, Entity as TableMetaEntity, Model,
    },
};

#[derive(Clone)]
pub struct DataSourceTableMetaRepository {
    db: DatabaseConnection,
}

impl DataSourceTableMetaRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// 物理删除该数据源下全部表/列元数据（刷新前清空）
    pub async fn delete_all_for_data_source(
        &self,
        data_source_id: i64,
    ) -> Result<(), sea_orm::DbErr> {
        let table_meta_ids = TableMetaEntity::find()
            .filter(Column::DataSourceId.eq(data_source_id))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|m| m.table_meta_id)
            .collect::<Vec<i64>>();

        if !table_meta_ids.is_empty() {
            ColumnMetaEntity::delete_many()
                .filter(ColumnMetaColumn::TableMetaId.is_in(table_meta_ids))
                .exec(&self.db)
                .await?;
        }

        TableMetaEntity::delete_many()
            .filter(Column::DataSourceId.eq(data_source_id))
            .exec(&self.db)
            .await?;

        Ok(())
    }

    /// 批量插入表元数据
    pub async fn insert_batch(
        &self,
        rows: &[DataSourceTableMeta],
    ) -> Result<(), sea_orm::DbErr> {
        if rows.is_empty() {
            return Ok(());
        }
        // MySQL prepared statement 最多 65535 个占位符；每行 12 列，
        // 分块 500 行（= 6000 占位符）避开 1390 错误。
        const CHUNK_SIZE: usize = 500;
        for chunk in rows.chunks(CHUNK_SIZE) {
            let active_models: Vec<ActiveModel> =
                chunk.iter().map(to_active_model).collect();
            TableMetaEntity::insert_many(active_models)
                .exec(&self.db)
                .await?;
        }
        Ok(())
    }

    pub async fn select_by_data_source_id(
        &self,
        data_source_id: &i64,
    ) -> Result<Vec<DataSourceTableMeta>, sea_orm::DbErr> {
        let models = TableMetaEntity::find()
            .filter(Column::DataSourceId.eq(*data_source_id))
            .filter(Column::DeletedBy.is_null())
            .filter(Column::DeletedDate.is_null())
            .order_by_asc(Column::TableName)
            .all(&self.db)
            .await?;
        Ok(models.into_iter().map(into_domain).collect())
    }

    pub async fn select_by_data_source_and_table(
        &self,
        data_source_id: &i64,
        table_name: &str,
    ) -> Result<Option<DataSourceTableMeta>, sea_orm::DbErr> {
        let model = TableMetaEntity::find()
            .filter(Column::DataSourceId.eq(*data_source_id))
            .filter(Column::TableName.eq(table_name))
            .filter(Column::DeletedBy.is_null())
            .filter(Column::DeletedDate.is_null())
            .one(&self.db)
            .await?;
        Ok(model.map(into_domain))
    }
}

fn to_active_model(row: &DataSourceTableMeta) -> ActiveModel {
    ActiveModel {
        table_meta_id: match row.table_meta_id {
            Some(id) => Set(id),
            None => NotSet,
        },
        data_source_id: Set(row.data_source_id),
        table_name: Set(row.table_name.clone()),
        comment_label: Set(row.comment_label.clone()),
        table_desc: Set(row.table_desc.clone()),
        version: Set(row.base_entity.version),
        created_by: Set(row.base_entity.created_by),
        created_date: Set(row.base_entity.created_date),
        last_modified_by: Set(row.base_entity.last_modified_by),
        last_modified_date: Set(row.base_entity.last_modified_date),
        deleted_by: Set(row.base_entity.deleted_by),
        deleted_date: Set(row.base_entity.deleted_date),
    }
}

fn into_domain(m: Model) -> DataSourceTableMeta {
    use crate::common::model::entity::BaseEntity;
    DataSourceTableMeta {
        table_meta_id: Some(m.table_meta_id),
        data_source_id: m.data_source_id,
        table_name: m.table_name,
        comment_label: m.comment_label,
        table_desc: m.table_desc,
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
