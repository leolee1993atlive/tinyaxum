use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::biz::httpapi::model::http_api::HttpApi;

#[derive(
    Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize,
)]
#[sea_orm(table_name = "biz_http_api")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub http_api_id: i64,
    pub http_api_name: String,
    pub http_api_path: String,
    pub http_api_desc: Option<String>,
    pub response_template: Option<String>,
    pub module: Option<String>,
    pub category: Option<String>,
    pub enabled: i32,
    pub default_params: Option<serde_json::Value>,
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

impl From<Model> for HttpApi {
    fn from(m: Model) -> Self {
        HttpApi {
            http_api_id: Some(m.http_api_id),
            http_api_name: m.http_api_name,
            http_api_path: m.http_api_path,
            http_api_desc: m.http_api_desc,
            response_template: m.response_template,
            module: m.module,
            category: m.category,
            enabled: m.enabled,
            default_params: m.default_params,
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

impl From<&HttpApi> for ActiveModel {
    fn from(api: &HttpApi) -> Self {
        ActiveModel {
            http_api_id: match api.http_api_id {
                Some(id) => sea_orm::ActiveValue::Set(id),
                None => sea_orm::ActiveValue::NotSet,
            },
            http_api_name: sea_orm::ActiveValue::Set(api.http_api_name.clone()),
            http_api_path: sea_orm::ActiveValue::Set(api.http_api_path.clone()),
            http_api_desc: sea_orm::ActiveValue::Set(api.http_api_desc.clone()),
            response_template: sea_orm::ActiveValue::Set(
                api.response_template.clone(),
            ),
            module: sea_orm::ActiveValue::Set(api.module.clone()),
            category: sea_orm::ActiveValue::Set(api.category.clone()),
            enabled: sea_orm::ActiveValue::Set(api.enabled),
            default_params: sea_orm::ActiveValue::Set(
                api.default_params.clone(),
            ),
            version: sea_orm::ActiveValue::Set(api.base_entity.version),
            created_by: sea_orm::ActiveValue::Set(api.base_entity.created_by),
            created_date: sea_orm::ActiveValue::Set(
                api.base_entity.created_date,
            ),
            last_modified_by: sea_orm::ActiveValue::Set(
                api.base_entity.last_modified_by,
            ),
            last_modified_date: sea_orm::ActiveValue::Set(
                api.base_entity.last_modified_date,
            ),
            deleted_by: sea_orm::ActiveValue::Set(api.base_entity.deleted_by),
            deleted_date: sea_orm::ActiveValue::Set(
                api.base_entity.deleted_date,
            ),
        }
    }
}
