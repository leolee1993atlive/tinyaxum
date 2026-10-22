use std::{sync::LazyLock, time::Instant};

use metrics::{counter, histogram};
use metrics_exporter_prometheus::PrometheusHandle;

static HANDLE: LazyLock<PrometheusHandle> = LazyLock::new(|| {
    let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
    let recorder = builder.build_recorder();
    let handle = recorder.handle();
    metrics::set_global_recorder(recorder)
        .expect("Failed to set metrics recorder");
    handle
});

pub fn init_metrics() {
    LazyLock::force(&HANDLE);
    tracing::info!("Prometheus metrics recorder 已安装");
}

pub fn metrics_output() -> String {
    HANDLE.render()
}

pub struct RequestTimer {
    method: String,
    path: String,
    start: Instant,
}

impl RequestTimer {
    pub fn new(method: String, path: String) -> Self {
        Self {
            method,
            path,
            start: Instant::now(),
        }
    }
}

impl Drop for RequestTimer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_secs_f64();
        histogram!(
            "http_request_duration_seconds",
            "method" => self.method.clone(),
            "path" => self.path.clone(),
        )
        .record(elapsed);
    }
}

pub struct RedisOpTimer {
    op: String,
    start: Instant,
}

impl RedisOpTimer {
    pub fn new(op: &str) -> Self {
        Self {
            op: op.to_string(),
            start: Instant::now(),
        }
    }
}

impl Drop for RedisOpTimer {
    fn drop(&mut self) {
        record_redis_op(&self.op, self.start.elapsed().as_secs_f64());
    }
}

pub struct DuckDbQueryTimer {
    start: Instant,
}

impl DuckDbQueryTimer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Drop for DuckDbQueryTimer {
    fn drop(&mut self) {
        record_duckdb_query_duration(self.start.elapsed().as_secs_f64());
    }
}

pub fn record_http_request(method: &str, path: &str, status: u16) {
    counter!(
        "http_requests_total",
        "method" => method.to_string(),
        "path" => path.to_string(),
        "status" => status.to_string(),
    )
    .increment(1);
}

pub fn record_redis_op(op: &str, duration_secs: f64) {
    histogram!(
        "redis_operation_duration_seconds",
        "op" => op.to_string(),
    )
    .record(duration_secs);
}

pub fn record_duckdb_query_duration(duration_secs: f64) {
    histogram!("duckdb_query_duration_seconds").record(duration_secs);
}
