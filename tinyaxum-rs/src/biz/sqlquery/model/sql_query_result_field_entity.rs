use sea_orm::entity::prelude::*;

use crate::biz::sqlquery::model::sql_query_result_field::SqlQueryResultField;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "biz_sql_query_result_field")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub result_field_id: i64,
    pub sql_query_id: i64,
    pub field_name: String,
    pub comment_label: Option<String>,
    pub field_type: String,
    pub format_pattern: Option<String>,
    pub field_desc: Option<String>,
    pub version: i64,
    pub created_by: i64,
    pub created_date: DateTimeUtc,
    pub last_modified_by: i64,
    pub last_modified_date: DateTimeUtc,
    pub deleted_by: Option<i64>,
    pub deleted_date: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for SqlQueryResultField {
    fn from(m: Model) -> Self {
        SqlQueryResultField {
            result_field_id: Some(m.result_field_id),
            sql_query_id: m.sql_query_id,
            field_name: m.field_name,
            comment_label: m.comment_label,
            field_type: m.field_type,
            format_pattern: m.format_pattern,
            field_desc: m.field_desc,
            base_entity: crate::common::model::entity::BaseEntity {
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

impl From<&SqlQueryResultField> for ActiveModel {
    fn from(r: &SqlQueryResultField) -> Self {
        use sea_orm::ActiveValue::{NotSet, Set};
        ActiveModel {
            result_field_id: match r.result_field_id {
                Some(id) => Set(id),
                None => NotSet,
            },
            sql_query_id: Set(r.sql_query_id),
            field_name: Set(r.field_name.clone()),
            comment_label: Set(r.comment_label.clone()),
            field_type: Set(r.field_type.clone()),
            format_pattern: Set(r.format_pattern.clone()),
            field_desc: Set(r.field_desc.clone()),
            version: Set(r.base_entity.version),
            created_by: Set(r.base_entity.created_by),
            created_date: Set(r.base_entity.created_date),
            last_modified_by: Set(r.base_entity.last_modified_by),
            last_modified_date: Set(r.base_entity.last_modified_date),
            deleted_by: Set(r.base_entity.deleted_by),
            deleted_date: Set(r.base_entity.deleted_date),
        }
    }
}
