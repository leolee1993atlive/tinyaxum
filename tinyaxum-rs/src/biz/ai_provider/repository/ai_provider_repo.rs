use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, TransactionTrait,
};

use crate::{
    biz::ai_provider::model::{
        ai_provider::AiProvider,
        ai_provider_entity::{
            ActiveModel, Column, Entity as BizAiProvider, Model,
        },
    },
    common::model::entity::BaseEntity,
};

impl From<&Model> for AiProvider {
    fn from(m: &Model) -> Self {
        AiProvider {
            ai_provider_id: Some(m.ai_provider_id),
            name: m.name.clone(),
            base_url: m.base_url.clone(),
            api_key: m.api_key.clone(),
            model: m.model.clone(),
            system_prompt: m.system_prompt.clone(),
            temperature: m.temperature.to_string().parse().unwrap_or(0.3),
            is_default: m.is_default,
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
}

fn to_active(p: &AiProvider) -> ActiveModel {
    ActiveModel {
        ai_provider_id: match p.ai_provider_id {
            Some(id) => Set(id),
            None => NotSet,
        },
        name: Set(p.name.clone()),
        base_url: Set(p.base_url.clone()),
        api_key: Set(p.api_key.clone()),
        model: Set(p.model.clone()),
        system_prompt: Set(p.system_prompt.clone()),
        temperature: Set(p
            .temperature
            .to_string()
            .parse()
            .unwrap_or(sea_orm::prelude::Decimal::ZERO)),
        is_default: Set(p.is_default),
        version: Set(p.base_entity.version),
        created_by: Set(p.base_entity.created_by),
        created_date: Set(p.base_entity.created_date),
        last_modified_by: Set(p.base_entity.last_modified_by),
        last_modified_date: Set(p.base_entity.last_modified_date),
        deleted_by: Set(p.base_entity.deleted_by),
        deleted_date: Set(p.base_entity.deleted_date),
    }
}

fn active_filter() -> Condition {
    Condition::all()
        .add(Column::DeletedBy.is_null())
        .add(Column::DeletedDate.is_null())
}

#[derive(Clone)]
pub struct AiProviderRepository {
    db: DatabaseConnection,
}

impl AiProviderRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn select_all(&self) -> Result<Vec<AiProvider>, sea_orm::DbErr> {
        let models = BizAiProvider::find()
            .filter(active_filter())
            .order_by_desc(Column::IsDefault)
            .order_by_asc(Column::AiProviderId)
            .all(&self.db)
            .await?;
        Ok(models.iter().map(AiProvider::from).collect())
    }

    pub async fn select_by_id(
        &self,
        id: i64,
    ) -> Result<Option<AiProvider>, sea_orm::DbErr> {
        let model = BizAiProvider::find_by_id(id)
            .filter(active_filter())
            .one(&self.db)
            .await?;
        Ok(model.as_ref().map(AiProvider::from))
    }

    pub async fn select_by_id_with_deleted(
        &self,
        id: i64,
    ) -> Result<Option<AiProvider>, sea_orm::DbErr> {
        let model = BizAiProvider::find_by_id(id).one(&self.db).await?;
        Ok(model.as_ref().map(AiProvider::from))
    }

    pub async fn select_default(
        &self,
    ) -> Result<Option<AiProvider>, sea_orm::DbErr> {
        let model = BizAiProvider::find()
            .filter(active_filter())
            .filter(Column::IsDefault.eq(1))
            .one(&self.db)
            .await?;
        Ok(model.as_ref().map(AiProvider::from))
    }

    pub async fn insert(&self, p: &AiProvider) -> Result<i64, sea_orm::DbErr> {
        let active = to_active(p);
        let res = BizAiProvider::insert(active).exec(&self.db).await?;
        Ok(res.last_insert_id)
    }

    pub async fn update_by_id(
        &self,
        p: &AiProvider,
    ) -> Result<(), sea_orm::DbErr> {
        let active = to_active(p);
        let _ = BizAiProvider::update(active)
            .validate()?
            .filter(active_filter())
            .exec(&self.db)
            .await?;
        Ok(())
    }

    pub async fn set_default_tx(
        &self,
        target_id: i64,
        modifier_user_id: i64,
    ) -> Result<(), sea_orm::DbErr> {
        let txn = self.db.begin().await?;
        BizAiProvider::update_many()
            .col_expr(Column::IsDefault, sea_orm::sea_query::Expr::value(0_i8))
            .col_expr(
                Column::LastModifiedBy,
                sea_orm::sea_query::Expr::value(modifier_user_id),
            )
            .col_expr(
                Column::LastModifiedDate,
                sea_orm::sea_query::Expr::value(chrono::Utc::now()),
            )
            .filter(active_filter())
            .exec(&txn)
            .await?;
        BizAiProvider::update_many()
            .col_expr(Column::IsDefault, sea_orm::sea_query::Expr::value(1_i8))
            .col_expr(
                Column::LastModifiedBy,
                sea_orm::sea_query::Expr::value(modifier_user_id),
            )
            .col_expr(
                Column::LastModifiedDate,
                sea_orm::sea_query::Expr::value(chrono::Utc::now()),
            )
            .filter(Column::AiProviderId.eq(target_id))
            .filter(active_filter())
            .exec(&txn)
            .await?;
        txn.commit().await?;
        Ok(())
    }
}
