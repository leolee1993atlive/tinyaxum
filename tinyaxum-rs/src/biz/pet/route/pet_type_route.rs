use std::sync::Arc;

use axum::{Router, routing::get};

use crate::{
    app::AppState,
    biz::pet::handler::pet_type_handler::{
        create_pet_type, delete_pet_type, get_pet_type_by_id, list_pet_type,
        update_pet_type,
    },
};

pub fn pet_type_route() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/pet_type",
            get(list_pet_type)
                .post(create_pet_type)
                .put(update_pet_type),
        )
        .route(
            "/pet_type/{id}",
            get(get_pet_type_by_id).delete(delete_pet_type),
        )
}
