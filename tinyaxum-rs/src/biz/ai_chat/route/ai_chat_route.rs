use std::sync::Arc;

use axum::{Router, routing::get};

use crate::{
    app::AppState,
    biz::ai_chat::handler::{
        ai_conversation_handler::{
            create_conversation, delete_conversation, list_conversations,
            update_conversation,
        },
        ai_message_handler::{list_messages, send_message},
    },
};

pub fn ai_chat_route() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/ai/conversations",
            get(list_conversations).post(create_conversation),
        )
        .route(
            "/ai/conversations/{id}",
            axum::routing::put(update_conversation).delete(delete_conversation),
        )
        .route(
            "/ai/conversations/{id}/messages",
            get(list_messages).post(send_message),
        )
}
