use std::sync::Arc;

use axum::Router;

use crate::{
    app::AppState, common::handler::module_option_handler::get_module_options,
};

pub fn module_option_route() -> Router<Arc<AppState>> {
    Router::new().route(
        "/common/module-options",
        axum::routing::get(get_module_options),
    )
}
