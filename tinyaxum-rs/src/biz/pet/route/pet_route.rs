use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get},
};

use crate::{
    app::AppState,
    biz::pet::handler::pet_handler::{
        create_pet, delete_pet, get_pet_by_id, list_pet, true_delete_pet,
        update_pet,
    },
};

pub fn pet_route() -> Router<Arc<AppState>> {
    Router::new()
        .route("/pet", get(list_pet).post(create_pet).put(update_pet))
        .route("/pet/{id}", get(get_pet_by_id).delete(delete_pet))
        .route("/pet/delete/{id}", delete(true_delete_pet))
}
