use std::sync::Arc;

use axum::{Router, routing::post};

use crate::{
    app::AppState,
    auth::handler::auth_handler::{login, logout, refresh_token},
};

pub fn auth_route() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/refreshToken", post(refresh_token))
        .route("/logout", post(logout))
}
