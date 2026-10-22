use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, Order,
    QueryFilter, QueryOrder,
};

use crate::{
    biz::ai_chat::model::{
        ai_conversation::AiConversation,
        ai_conversation_entity::{
            ActiveModel, Column, Entity as BizAiConv, Model,
        },
    },
    common::model::entity::BaseEntity,
};

impl From<&Model> for AiConversation {
    fn from(m: &Model) -> Self {
        AiConversation {
            ai_conversation_id: Some(m.ai_conversation_id),
            sql_query_id: m.sql_query_id,
            ai_provider_id: m.ai_provider_id,
            title: m.title.clone(),
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

fn to_active(c: &AiConversation) -> ActiveModel {
    ActiveModel {
        ai_conversation_id: match c.ai_conversation_id {
            Some(id) => Set(id),
            None => NotSet,
        },
        sql_query_id: Set(c.sql_query_id),
        ai_provider_id: Set(c.ai_provider_id),
        title: Set(c.title.clone()),
        version: Set(c.base_entity.version),
        created_by: Set(c.base_entity.created_by),
        created_date: Set(c.base_entity.created_date),
        last_modified_by: Set(c.base_entity.last_modified_by),
        last_modified_date: Set(c.base_entity.last_modified_date),
        deleted_by: Set(c.base_entity.deleted_by),
        deleted_date: Set(c.base_entity.deleted_date),
    }
}

fn active_filter() -> Condition {
    Condition::all()
        .add(Column::DeletedBy.is_null())
        .add(Column::DeletedDate.is_null())
}

#[derive(Clone)]
pub struct AiConversationRepository {
    db: DatabaseConnection,
}

impl AiConversationRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn select_by_sql_query(
        &self,
        sql_query_id: i64,
    ) -> Result<Vec<AiConversation>, sea_orm::DbErr> {
        let models = BizAiConv::find()
            .filter(active_filter())
            .filter(Column::SqlQueryId.eq(sql_query_id))
            .order_by(Column::AiConversationId, Order::Desc)
            .all(&self.db)
            .await?;
        Ok(models.iter().map(AiConversation::from).collect())
    }

    pub async fn select_by_id(
        &self,
        id: i64,
    ) -> Result<Option<AiConversation>, sea_orm::DbErr> {
        let model = BizAiConv::find_by_id(id)
            .filter(active_filter())
            .one(&self.db)
            .await?;
        Ok(model.as_ref().map(AiConversation::from))
    }

    pub async fn insert(
        &self,
        c: &AiConversation,
    ) -> Result<i64, sea_orm::DbErr> {
        let active = to_active(c);
        let res = BizAiConv::insert(active).exec(&self.db).await?;
        Ok(res.last_insert_id)
    }

    pub async fn update_by_id(
        &self,
        c: &AiConversation,
    ) -> Result<(), sea_orm::DbErr> {
        let active = to_active(c);
        let _ = BizAiConv::update(active)
            .validate()?
            .filter(active_filter())
            .exec(&self.db)
            .await?;
        Ok(())
    }
}
