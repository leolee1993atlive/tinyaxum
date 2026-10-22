use crate::{
    app::Infrastructure,
    biz::{
        ai_chat::{
            model::ai_message::AiMessage,
            repository::ai_message_repo::AiMessageRepository,
            service::ai_conversation_service::AiConversationService,
        },
        ai_provider::service::ai_provider_service::AiProviderService,
        datasource::service::data_source_service::DataSourceService,
        sqlquery::service::sql_query_service::SqlQueryService,
    },
    error::error::AppResult,
    util::ai_client::ChatMessage,
};

#[derive(Clone)]
pub struct AiMessageService {
    pub message_repo: AiMessageRepository,
    pub provider_service: AiProviderService,
    pub conversation_service: AiConversationService,
    pub data_source_service: DataSourceService,
    pub sql_query_service: SqlQueryService,
}

impl AiMessageService {
    pub fn new(
        infra: &Infrastructure,
        provider_service: AiProviderService,
        conversation_service: AiConversationService,
        data_source_service: DataSourceService,
        sql_query_service: SqlQueryService,
    ) -> Self {
        Self {
            message_repo: AiMessageRepository::new(infra.db.clone()),
            provider_service,
            conversation_service,
            data_source_service,
            sql_query_service,
        }
    }

    pub async fn list_messages(
        &self,
        conv_id: i64,
    ) -> AppResult<Vec<AiMessage>> {
        Ok(self.message_repo.select_by_conversation(conv_id).await?)
    }

    pub async fn fetch_schema_text_for_ds(
        &self,
        data_source_id: i64,
    ) -> AppResult<String> {
        let schema =
            self.data_source_service.get_schema(data_source_id).await?;
        let mut out = String::new();
        for t in schema.tables.iter() {
            out.push_str(&t.table_name);
            out.push_str(" (");
            for (i, c) in t.columns.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&c.column_name);
                out.push(' ');
                out.push_str(&c.field_type);
            }
            out.push_str(")\n");
        }
        Ok(out)
    }

    pub fn send_message_stream(
        self,
        conv_id: i64,
        bo: crate::biz::ai_chat::model::ai_message::SendMessageBo,
        user: crate::middleware::extractors::CurrentUser,
    ) -> impl futures_util::Stream<
        Item = Result<axum::response::sse::Event, std::convert::Infallible>,
    > {
        use axum::response::sse::Event;
        use futures_util::StreamExt;
        use serde_json::json;

        use crate::{
            biz::ai_chat::model::ai_message::AiMessage,
            util::ai_client::chat_completion_stream,
        };

        async_stream::stream! {
            let conv = match self.conversation_service.get_by_id(conv_id).await {
                Ok(c) => c,
                Err(e) => {
                    yield Ok(Event::default()
                        .event("error")
                        .data(json!({"code":"CONVERSATION_NOT_FOUND","message":e.to_string()}).to_string()));
                    return;
                }
            };

            let (provider, plain_key) = match self
                .provider_service
                .get_decrypted_for_call(conv.ai_provider_id)
                .await
            {
                Ok(t) => t,
                Err(e) => {
                    let msg = e.to_string();
                    let code = if msg.contains("PROVIDER_DELETED") {
                        "PROVIDER_DELETED"
                    } else {
                        "PROVIDER_ERROR"
                    };
                    yield Ok(Event::default()
                        .event("error")
                        .data(json!({"code":code,"message":msg}).to_string()));
                    return;
                }
            };

            let history = match self.message_repo.select_by_conversation(conv_id).await {
                Ok(h) => h,
                Err(e) => {
                    yield Ok(Event::default()
                        .event("error")
                        .data(json!({"code":"DB_ERROR","message":e.to_string()}).to_string()));
                    return;
                }
            };

            let schema_text = if bo.attach_schema {
                let ds_id = match self
                    .sql_query_service
                    .get_sql_query_by_id(conv.sql_query_id)
                    .await
                {
                    Ok(detail) => detail.data_source_id,
                    Err(_) => 0,
                };
                if ds_id > 0 {
                    match self.fetch_schema_text_for_ds(ds_id).await {
                        Ok(t) => Some(t),
                        Err(e) => Some(format!("(schema 获取失败: {})", e)),
                    }
                } else {
                    Some("(无关联数据源)".to_string())
                }
            } else {
                None
            };

            let user_msg = AiMessage::user_msg(
                conv_id,
                bo.content.clone(),
                bo.attach_schema,
                &user,
            );
            let user_msg_id = match self.message_repo.insert(&user_msg).await {
                Ok(id) => id,
                Err(e) => {
                    yield Ok(Event::default()
                        .event("error")
                        .data(json!({"code":"DB_ERROR","message":e.to_string()}).to_string()));
                    return;
                }
            };
            yield Ok(Event::default()
                .event("user_saved")
                .data(json!({"id":user_msg_id}).to_string()));

            let augmented = augment_user_content(
                &bo.content,
                bo.current_sql.as_deref(),
                schema_text.as_deref(),
            );
            let msgs = build_upstream_messages(&provider.system_prompt, &history, &augmented);

            let mut stream = std::pin::pin!(chat_completion_stream(
                provider.base_url.clone(),
                plain_key,
                provider.model.clone(),
                msgs,
                provider.temperature,
            ));

            let mut accumulated = String::new();
            let mut last_usage: Option<crate::util::ai_client::TokenUsage> = None;

            while let Some(item) = stream.next().await {
                match item {
                    Ok(chunk) => {
                        if !chunk.delta_content.is_empty() {
                            accumulated.push_str(&chunk.delta_content);
                            yield Ok(Event::default()
                                .event("delta")
                                .data(json!({"content":chunk.delta_content}).to_string()));
                        }
                        if let Some(u) = chunk.usage {
                            last_usage = Some(u);
                        }
                        if chunk.done {
                            break;
                        }
                    }
                    Err(e) => {
                        yield Ok(Event::default()
                            .event("error")
                            .data(json!({"code":"UPSTREAM_ERROR","message":e.to_string()}).to_string()));
                        return;
                    }
                }
            }

            let assistant_msg = AiMessage::assistant_msg(
                conv_id,
                accumulated,
                last_usage.as_ref().map(|u| u.prompt_tokens),
                last_usage.as_ref().map(|u| u.completion_tokens),
                &user,
            );
            let assistant_id = match self.message_repo.insert(&assistant_msg).await {
                Ok(id) => id,
                Err(e) => {
                    yield Ok(Event::default()
                        .event("error")
                        .data(json!({"code":"DB_ERROR","message":format!("assistant 消息落库失败: {e}")}).to_string()));
                    return;
                }
            };
            tracing::info!(
                conversation_id = conv_id,
                provider_id = conv.ai_provider_id,
                prompt_tokens = last_usage.as_ref().map(|u| u.prompt_tokens).unwrap_or(0),
                completion_tokens = last_usage.as_ref().map(|u| u.completion_tokens).unwrap_or(0),
                "AI chat message completed"
            );
            yield Ok(Event::default()
                .event("done")
                .data(json!({
                    "id": assistant_id,
                    "promptTokens": last_usage.as_ref().map(|u| u.prompt_tokens),
                    "completionTokens": last_usage.as_ref().map(|u| u.completion_tokens),
                }).to_string()));
        }
    }
}

pub fn augment_user_content(
    user_input: &str,
    current_sql: Option<&str>,
    schema_text: Option<&str>,
) -> String {
    let mut s = user_input.to_string();
    if let Some(sql) = current_sql.filter(|s| !s.trim().is_empty()) {
        s.push_str("\n\n---\n当前 SQL:\n```sql\n");
        s.push_str(sql);
        s.push_str("\n```");
    }
    if let Some(schema) = schema_text.filter(|s| !s.trim().is_empty()) {
        s.push_str("\n\n---\n相关表结构:\n");
        s.push_str(schema);
    }
    s
}

pub fn build_upstream_messages(
    system_prompt: &str,
    history: &[AiMessage],
    augmented_user_content: &str,
) -> Vec<ChatMessage> {
    let mut out = Vec::with_capacity(history.len() + 2);
    out.push(ChatMessage {
        role: "system".to_string(),
        content: system_prompt.to_string(),
    });
    for m in history.iter() {
        if m.role == "user" || m.role == "assistant" {
            out.push(ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            });
        }
    }
    out.push(ChatMessage {
        role: "user".to_string(),
        content: augmented_user_content.to_string(),
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::model::entity::BaseEntity;

    fn fake_msg(role: &str, content: &str) -> AiMessage {
        AiMessage {
            ai_message_id: Some(1),
            ai_conversation_id: 1,
            role: role.to_string(),
            content: content.to_string(),
            attached_schema: 0,
            prompt_tokens: None,
            completion_tokens: None,
            base_entity: BaseEntity::new(1),
        }
    }

    #[test]
    fn build_upstream_messages_no_attach() {
        let augmented = augment_user_content("hi", None, None);
        let msgs = build_upstream_messages("you are X", &[], &augmented);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].role, "system");
        assert_eq!(msgs[0].content, "you are X");
        assert_eq!(msgs[1].role, "user");
        assert_eq!(msgs[1].content, "hi");
    }

    #[test]
    fn build_upstream_messages_with_sql() {
        let augmented =
            augment_user_content("rewrite this", Some("SELECT 1"), None);
        let msgs = build_upstream_messages("you are X", &[], &augmented);
        assert_eq!(msgs.len(), 2);
        assert!(msgs[1].content.contains("rewrite this"));
        assert!(msgs[1].content.contains("```sql"));
        assert!(msgs[1].content.contains("SELECT 1"));
    }

    #[test]
    fn build_upstream_messages_with_schema() {
        let augmented = augment_user_content(
            "?",
            None,
            Some("orders (id INT, amt DECIMAL)"),
        );
        let msgs = build_upstream_messages("you are X", &[], &augmented);
        assert!(msgs[1].content.contains("相关表结构"));
        assert!(msgs[1].content.contains("orders (id INT"));
    }

    #[test]
    fn build_upstream_messages_system_from_provider() {
        let history =
            vec![fake_msg("user", "previous"), fake_msg("assistant", "ok")];
        let augmented = augment_user_content("now", None, None);
        let msgs =
            build_upstream_messages("CUSTOM_SYSTEM", &history, &augmented);
        assert_eq!(msgs.len(), 4);
        assert_eq!(msgs[0].role, "system");
        assert_eq!(msgs[0].content, "CUSTOM_SYSTEM");
        assert_eq!(msgs[1].role, "user");
        assert_eq!(msgs[1].content, "previous");
        assert_eq!(msgs[2].role, "assistant");
        assert_eq!(msgs[3].role, "user");
        assert_eq!(msgs[3].content, "now");
    }

    #[test]
    fn augment_user_content_strips_empty_optionals() {
        let s = augment_user_content("hello", Some("   "), Some(""));
        assert_eq!(s, "hello");
    }
}
