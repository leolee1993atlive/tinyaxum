use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post, put},
};

use crate::{
    app::AppState,
    biz::ai_provider::handler::ai_provider_handler::{
        create_provider, delete_provider, get_provider, list_providers,
        set_default_provider, test_provider, update_provider,
    },
};

pub fn ai_provider_route() -> Router<Arc<AppState>> {
    Router::new()
        .route("/ai/providers", get(list_providers).post(create_provider))
        .route(
            "/ai/providers/{id}",
            get(get_provider)
                .put(update_provider)
                .delete(delete_provider),
        )
        .route("/ai/providers/{id}/test", post(test_provider))
        .route("/ai/providers/{id}/default", put(set_default_provider))
}
