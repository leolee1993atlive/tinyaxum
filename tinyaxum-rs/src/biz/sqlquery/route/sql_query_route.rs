use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::{
    app::AppState,
    biz::sqlquery::handler::sql_query_handler::{
        create_sql_query, delete_sql_query, get_query_schema,
        get_sql_query_by_id, list_sql_query, list_sql_query_page,
        run_adhoc_sql_query, run_sql_query, save_sql_query_viz_config,
        true_delete_sql_query, update_sql_query, validate_sql_query,
    },
};

pub fn sql_query_route() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/sqlquery",
            get(list_sql_query)
                .post(create_sql_query)
                .put(update_sql_query),
        )
        .route("/sqlquery/page", get(list_sql_query_page))
        .route("/sqlquery/{id}/viz-config", put(save_sql_query_viz_config))
        .route(
            "/sqlquery/{id}",
            get(get_sql_query_by_id).delete(delete_sql_query),
        )
        .route("/sqlquery/delete/{id}", delete(true_delete_sql_query))
        .route("/sqlquery/run/{id}", get(run_sql_query))
        .route("/sqlquery/run-adhoc", post(run_adhoc_sql_query))
        .route("/sqlquery/validate", post(validate_sql_query))
        .route("/sqlquery/schema/{dataSourceId}", get(get_query_schema))
}
