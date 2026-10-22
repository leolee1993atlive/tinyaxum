use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, Order,
    QueryFilter, QueryOrder,
};

use crate::{
    biz::ai_chat::model::{
        ai_message::AiMessage,
        ai_message_entity::{ActiveModel, Column, Entity as BizAiMsg, Model},
    },
    common::model::entity::BaseEntity,
};

impl From<&Model> for AiMessage {
    fn from(m: &Model) -> Self {
        AiMessage {
            ai_message_id: Some(m.ai_message_id),
            ai_conversation_id: m.ai_conversation_id,
            role: m.role.clone(),
            content: m.content.clone(),
            attached_schema: m.attached_schema,
            prompt_tokens: m.prompt_tokens,
            completion_tokens: m.completion_tokens,
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

fn to_active(m: &AiMessage) -> ActiveModel {
    ActiveModel {
        ai_message_id: match m.ai_message_id {
            Some(id) => Set(id),
            None => NotSet,
        },
        ai_conversation_id: Set(m.ai_conversation_id),
        role: Set(m.role.clone()),
        content: Set(m.content.clone()),
        attached_schema: Set(m.attached_schema),
        prompt_tokens: Set(m.prompt_tokens),
        completion_tokens: Set(m.completion_tokens),
        version: Set(m.base_entity.version),
        created_by: Set(m.base_entity.created_by),
        created_date: Set(m.base_entity.created_date),
        last_modified_by: Set(m.base_entity.last_modified_by),
        last_modified_date: Set(m.base_entity.last_modified_date),
        deleted_by: Set(m.base_entity.deleted_by),
        deleted_date: Set(m.base_entity.deleted_date),
    }
}

fn active_filter() -> Condition {
    Condition::all()
        .add(Column::DeletedBy.is_null())
        .add(Column::DeletedDate.is_null())
}

#[derive(Clone)]
pub struct AiMessageRepository {
    db: DatabaseConnection,
}

impl AiMessageRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn select_by_conversation(
        &self,
        conv_id: i64,
    ) -> Result<Vec<AiMessage>, sea_orm::DbErr> {
        let models = BizAiMsg::find()
            .filter(active_filter())
            .filter(Column::AiConversationId.eq(conv_id))
            .order_by(Column::AiMessageId, Order::Asc)
            .all(&self.db)
            .await?;
        Ok(models.iter().map(AiMessage::from).collect())
    }

    pub async fn insert(&self, m: &AiMessage) -> Result<i64, sea_orm::DbErr> {
        let active = to_active(m);
        let res = BizAiMsg::insert(active).exec(&self.db).await?;
        Ok(res.last_insert_id)
    }
}
