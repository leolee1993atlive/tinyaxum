use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::{
    app::AppState,
    biz::transtask::handler::trans_task_handler::{
        create_trans_task, create_trans_task_batch, delete_trans_task,
        get_trans_task_by_id, list_trans_task, list_trans_task_page,
        run_trans_task_sse, start_trans_task, stop_trans_task, test_trans_task,
        transtask_events_sse, true_delete_trans_task, update_trans_task,
    },
};

pub fn trans_task_route() -> Router<Arc<AppState>> {
    Router::new()
        .route("/transtask/batch", post(create_trans_task_batch))
        .route(
            "/transtask",
            get(list_trans_task)
                .post(create_trans_task)
                .put(update_trans_task),
        )
        .route("/transtask/page", get(list_trans_task_page))
        .route(
            "/transtask/{id}",
            get(get_trans_task_by_id).delete(delete_trans_task),
        )
        .route("/transtask/delete/{id}", delete(true_delete_trans_task))
        .route("/transtask/test/{id}", post(test_trans_task))
        .route("/transtask/run/{id}", post(run_trans_task_sse))
        .route("/transtask/start/{id}", post(start_trans_task))
        .route("/transtask/stop/{id}", post(stop_trans_task))
        .route("/transtask/events", get(transtask_events_sse))
}
