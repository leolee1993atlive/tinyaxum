use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, put},
};

use crate::{
    app::AppState,
    sys::user::handler::user_handler::{
        create_user, delete_user, get_user_by_id, get_user_info, list_user,
        list_user_page, true_delete_user, update_password, update_user,
    },
};

pub fn user_route() -> Router<Arc<AppState>> {
    Router::new()
        .route("/user", get(list_user).post(create_user).put(update_user))
        .route("/user/page", get(list_user_page))
        .route("/user/{id}", get(get_user_by_id).delete(delete_user))
        .route("/user/delete/{id}", delete(true_delete_user))
        .route("/user/password", put(update_password))
        .route("/user/info", get(get_user_info))
}
