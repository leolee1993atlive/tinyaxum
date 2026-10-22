use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get},
};

use crate::{
    app::AppState,
    biz::sqlquery::handler::sql_snippet_handler::{
        create_sql_snippet, delete_sql_snippet, get_sql_snippet_by_id,
        list_sql_snippet, list_sql_snippet_page, true_delete_sql_snippet,
        update_sql_snippet,
    },
};

pub fn sql_snippet_route() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/sqlsnippet",
            get(list_sql_snippet)
                .post(create_sql_snippet)
                .put(update_sql_snippet),
        )
        .route("/sqlsnippet/page", get(list_sql_snippet_page))
        .route(
            "/sqlsnippet/{id}",
            get(get_sql_snippet_by_id).delete(delete_sql_snippet),
        )
        .route("/sqlsnippet/delete/{id}", delete(true_delete_sql_snippet))
}
