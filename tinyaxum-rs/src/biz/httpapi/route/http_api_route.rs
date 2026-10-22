use std::sync::Arc;

use axum::{
    Router,
    routing::{get, put},
};

use crate::{app::AppState, biz::httpapi::handler::http_api_handler::*};

pub fn http_api_route() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/httpapi",
            get(list_http_api)
                .post(create_http_api)
                .put(update_http_api),
        )
        .route("/httpapi/page", get(get_http_api_page))
        .route(
            "/httpapi/{id}",
            get(get_http_api_by_id).delete(delete_http_api),
        )
        .route("/httpapi/{id}/enable", put(enable_http_api))
        .route("/httpapi/{id}/disable", put(disable_http_api))
}

pub fn public_route() -> Router<Arc<AppState>> {
    Router::new().route("/public/{*path}", get(execute_public_api))
}
