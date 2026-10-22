use sea_orm::entity::prelude::*;

use crate::biz::sqlquery::model::sql_snippet::SqlSnippet;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "biz_sql_snippet")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub sql_snippet_id: i64,
    pub sql_snippet_name: String,
    pub sql_snippet_content: String,
    pub sql_snippet_desc: Option<String>,
    pub sql_snippet_tag: Option<String>,
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

impl From<Model> for SqlSnippet {
    fn from(m: Model) -> Self {
        SqlSnippet {
            sql_snippet_id: Some(m.sql_snippet_id),
            sql_snippet_name: m.sql_snippet_name,
            sql_snippet_content: m.sql_snippet_content,
            sql_snippet_desc: m.sql_snippet_desc,
            sql_snippet_tag: m.sql_snippet_tag,
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

impl From<&SqlSnippet> for ActiveModel {
    fn from(s: &SqlSnippet) -> Self {
        use sea_orm::ActiveValue::{NotSet, Set};
        ActiveModel {
            sql_snippet_id: match s.sql_snippet_id {
                Some(id) => Set(id),
                None => NotSet,
            },
            sql_snippet_name: Set(s.sql_snippet_name.clone()),
            sql_snippet_content: Set(s.sql_snippet_content.clone()),
            sql_snippet_desc: Set(s.sql_snippet_desc.clone()),
            sql_snippet_tag: Set(s.sql_snippet_tag.clone()),
            version: Set(s.base_entity.version),
            created_by: Set(s.base_entity.created_by),
            created_date: Set(s.base_entity.created_date),
            last_modified_by: Set(s.base_entity.last_modified_by),
            last_modified_date: Set(s.base_entity.last_modified_date),
            deleted_by: Set(s.base_entity.deleted_by),
            deleted_date: Set(s.base_entity.deleted_date),
        }
    }
}
