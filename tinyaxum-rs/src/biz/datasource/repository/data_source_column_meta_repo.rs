use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};

use crate::biz::datasource::model::{
    data_source_column_meta::DataSourceColumnMeta,
    data_source_column_meta_entity::{
        ActiveModel, Column, Entity as ColumnMetaEntity, Model,
    },
};

#[derive(Clone)]
pub struct DataSourceColumnMetaRepository {
    db: DatabaseConnection,
}

impl DataSourceColumnMetaRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// 批量插入列元数据
    pub async fn insert_batch(
        &self,
        rows: &[DataSourceColumnMeta],
    ) -> Result<(), sea_orm::DbErr> {
        if rows.is_empty() {
            return Ok(());
        }
        // MySQL prepared statement 最多 65535 个占位符；每行 16 列，
        // 分块 500 行（= 8000 占位符）避开 1390 错误。
        const CHUNK_SIZE: usize = 500;
        for chunk in rows.chunks(CHUNK_SIZE) {
            let active_models: Vec<ActiveModel> =
                chunk.iter().map(to_active_model).collect();
            ColumnMetaEntity::insert_many(active_models)
                .exec(&self.db)
                .await?;
        }
        Ok(())
    }

    /// 一次性查出多张表的列元数据，调用方按 table_meta_id 分组，避免 N+1
    pub async fn select_by_table_meta_ids(
        &self,
        table_meta_ids: &[i64],
    ) -> Result<Vec<DataSourceColumnMeta>, sea_orm::DbErr> {
        if table_meta_ids.is_empty() {
            return Ok(Vec::new());
        }
        let models = ColumnMetaEntity::find()
            .filter(Column::TableMetaId.is_in(table_meta_ids.iter().copied()))
            .filter(Column::DeletedBy.is_null())
            .filter(Column::DeletedDate.is_null())
            .order_by_asc(Column::TableMetaId)
            .order_by_asc(Column::OrdinalPosition)
            .order_by_asc(Column::ColumnName)
            .all(&self.db)
            .await?;
        Ok(models.into_iter().map(into_domain).collect())
    }

    /// 查询指定表中标记为主键的列（有序），用于优先级链中的元数据 PK 解析
    pub async fn select_pk_by_table_meta_id(
        &self,
        table_meta_id: &i64,
    ) -> Result<Vec<DataSourceColumnMeta>, sea_orm::DbErr> {
        let models = ColumnMetaEntity::find()
            .filter(Column::TableMetaId.eq(*table_meta_id))
            .filter(Column::IsPrimaryKey.eq(1))
            .filter(Column::DeletedBy.is_null())
            .filter(Column::DeletedDate.is_null())
            .order_by_asc(Column::OrdinalPosition)
            .all(&self.db)
            .await?;
        Ok(models.into_iter().map(into_domain).collect())
    }
}

fn to_active_model(row: &DataSourceColumnMeta) -> ActiveModel {
    ActiveModel {
        column_meta_id: match row.column_meta_id {
            Some(id) => Set(id),
            None => NotSet,
        },
        table_meta_id: Set(row.table_meta_id),
        column_name: Set(row.column_name.clone()),
        comment_label: Set(row.comment_label.clone()),
        field_type: Set(row.field_type.clone()),
        format_pattern: Set(row.format_pattern.clone()),
        field_desc: Set(row.field_desc.clone()),
        source_physical_type: Set(row.source_physical_type.clone()),
        ordinal_position: Set(row.ordinal_position),
        is_primary_key: Set(row.is_primary_key),
        version: Set(row.base_entity.version),
        created_by: Set(row.base_entity.created_by),
        created_date: Set(row.base_entity.created_date),
        last_modified_by: Set(row.base_entity.last_modified_by),
        last_modified_date: Set(row.base_entity.last_modified_date),
        deleted_by: Set(row.base_entity.deleted_by),
        deleted_date: Set(row.base_entity.deleted_date),
    }
}

fn into_domain(m: Model) -> DataSourceColumnMeta {
    use crate::common::model::entity::BaseEntity;
    DataSourceColumnMeta {
        column_meta_id: Some(m.column_meta_id),
        table_meta_id: m.table_meta_id,
        column_name: m.column_name,
        comment_label: m.comment_label,
        field_type: m.field_type,
        format_pattern: m.format_pattern,
        field_desc: m.field_desc,
        source_physical_type: m.source_physical_type,
        ordinal_position: m.ordinal_position,
        is_primary_key: m.is_primary_key,
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
