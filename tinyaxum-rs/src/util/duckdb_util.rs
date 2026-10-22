use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use chrono::TimeZone as _;
use duckdb::Connection;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::mpsc;
use tracing::info;

use crate::biz::{
    datasource::model::data_source::DataSource,
    transtask::model::trans_task::TransTaskDetailVo,
};

// ── 进度事件 ─────────────────────────────────────────────────────────────────

/// anyhow 完整错误链（单行 "外层: 内层: 根因"）。
/// `%e` 只显示最外层 context，生产排障时看不到底层 DuckDB/驱动原始报错。
/// 对非 anyhow 的根错误（如 duckdb::Error）退化为自身 Display。
fn err_chain(e: &impl std::fmt::Display) -> String {
    format!("{:#}", e)
}

/// 同步过程中推送给前端的进度事件（通过 SSE 传输）
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SyncProgressEvent {
    /// 正在写入(全量初始阶段)
    Exporting { phase: String },
    /// 分批写入进度
    #[serde(rename_all = "camelCase")]
    Progress { rows_done: u64 },
    /// 同步完成
    #[serde(rename_all = "camelCase")]
    Done {
        row_count: i64,
        new_watermark: Option<String>,
    },
    /// 同步出错
    Error { message: String },
}

/// 进度发送端别名（使用 blocking_send 从同步线程发送）
pub type ProgressTx = mpsc::Sender<SyncProgressEvent>;

/// 在同步线程中发送进度事件（忽略发送失败，连接断开属正常情况）
fn escape_sql_string(val: &str) -> String {
    val.replace('\'', "''")
}

fn emit(tx: &Option<ProgressTx>, event: SyncProgressEvent) {
    if let Some(tx) = tx {
        let _ = tx.blocking_send(event);
    }
}

pub fn validate_identifier(name: &str) -> Result<()> {
    let forbidden = [';', '\'', '"', '/', '*', '#', '`', '\\'];
    if name.is_empty() {
        return Err(anyhow::anyhow!("标识符不能为空"));
    }
    if name.chars().any(|c| forbidden.contains(&c)) {
        return Err(anyhow::anyhow!("无效标识符 '{}': 包含非法字符", name));
    }
    if name.contains("--") {
        return Err(anyhow::anyhow!(
            "无效标识符 '{}': 包含 SQL 注释序列",
            name
        ));
    }
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => {
            return Err(anyhow::anyhow!(
                "无效标识符 '{}': 必须以字母或下划线开头",
                name
            ));
        }
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Err(anyhow::anyhow!(
            "无效标识符 '{}': 只能包含字母、数字、下划线或连字符",
            name
        ));
    }
    Ok(())
}

/// 同步结果：行数 + 可选的新水位值
pub struct SyncResult {
    pub row_count: usize,
    pub new_watermark: Option<String>,
}

pub fn trans_table(
    data_source: &DataSource,
    trans_task: &TransTaskDetailVo,
    data_dir: &std::path::Path,
    progress_tx: Option<ProgressTx>,
) -> Result<SyncResult> {
    let db_type = data_source.db_type.to_lowercase();

    info!(
        db_type = %db_type,
        db_name = %data_source.db_name,
        table_name = %trans_task.table_name,
        sync_mode = %trans_task.sync_mode,
        "开始数据传输任务"
    );

    let db_path = data_dir
        .join(format!("src_{}", data_source.code))
        .join(format!("{}.duckdb", trans_task.table_name));

    let syncer = DbToDuckdbSync::new(
        &db_type,
        &data_source.db_host,
        data_source.db_port,
        &data_source.db_name,
        &data_source.db_username,
        &data_source.db_password,
        data_source.db_schema.as_deref(),
        &db_path,
    )
    .inspect_err(|e| {
        tracing::error!(
            error = %err_chain(e),
            db_name = %data_source.db_name,
            "初始化同步器失败"
        );
    })?;

    let sync_result = match trans_task.sync_mode.as_str() {
        "incremental" | "realtime" => {
            let sync_strategy =
                trans_task.sync_strategy.as_deref().unwrap_or("timestamp");
            let sync_column = trans_task.sync_column.as_deref().unwrap_or("");

            if sync_column.is_empty() {
                return Err(anyhow::anyhow!(
                    "增量/实时同步模式下 sync_column 不能为空"
                ));
            }

            syncer
                .incremental_sync(
                    &data_source.code,
                    &data_source.db_name,
                    &trans_task.table_name,
                    sync_strategy,
                    sync_column,
                    trans_task.sync_watermark.as_deref(),
                    trans_task.sync_primary_key.as_deref(),
                    &progress_tx,
                )
                .context(format!(
                    "增量同步数据源 '{}' 的表 '{}' 失败",
                    data_source.code, trans_task.table_name
                ))
                .inspect_err(|e| {
                    tracing::error!(
                        error = %err_chain(e),
                        db_name = %data_source.db_name,
                        table_name = %trans_task.table_name,
                        "增量同步失败"
                    );
                })?
        }
        _ => {
            // 全量同步走纯流式：不需要主键，DuckDB scanner 自带流式拉取
            let row_count = syncer
                .full_sync(
                    &data_source.code,
                    &data_source.db_name,
                    &trans_task.table_name,
                    &progress_tx,
                )
                .context(format!(
                    "全量同步数据源 '{}' 的表 '{}' 到 DuckDB 失败",
                    data_source.code, trans_task.table_name
                ))
                .inspect_err(|e| {
                    tracing::error!(
                        error = %err_chain(e),
                        db_name = %data_source.db_name,
                        table_name = %trans_task.table_name,
                        "全量同步失败"
                    );
                })?;
            SyncResult {
                row_count,
                new_watermark: None,
            }
        }
    };

    info!(
        db_name = %data_source.db_name,
        table_name = %trans_task.table_name,
        rows = sync_result.row_count,
        new_watermark = ?sync_result.new_watermark,
        "数据传输任务完成"
    );

    Ok(sync_result)
}

pub struct DbToDuckdbSync {
    conn: Arc<Mutex<Connection>>,
    db_type: String,
    /// 源库显式 schema（仅 PG 使用，缺省 public）
    schema: Option<String>,
}

impl DbToDuckdbSync {
    /// 打开（或创建）每表专属 DuckDB 文件，安装并附加源数据库。
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db_type: &str,
        db_host: &str,
        db_port: u16,
        db_name: &str,
        db_user: &str,
        db_pass: &str,
        db_schema: Option<&str>,
        db_path: &std::path::Path,
    ) -> Result<Self> {
        if let Some(schema) = db_schema.filter(|s| !s.is_empty()) {
            validate_identifier(schema)
                .context(format!("无效的 schema 名: {}", schema))?;
        }
        // 确保目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("创建目录 '{:?}' 失败", parent))?;
        }

        info!(path = %db_path.display(), "正在打开每表 DuckDB 文件");
        let conn = Connection::open(db_path)
            .with_context(|| format!("打开DuckDB文件失败: {:?}", db_path))?;

        let (ext_file, ext_short) = match db_type {
            "mysql" => ("mysql_scanner", "mysql"),
            "postgres" | "postgresql" => ("postgres_scanner", "postgres"),
            other => {
                return Err(anyhow::anyhow!("不支持的数据库类型: {}", other));
            }
        };
        // 离线优先：若 exe 同目录下 extensions/<ext_file>.duckdb_extension 存在，直接从
        // 本地文件 LOAD（生产机无外网时走这条）；否则回退到在线 INSTALL+LOAD（开发
        // 环境）。所有平台统一策略：链接官方预编译 libduckdb，与官方预编译扩展 ABI
        // 兼容。官方扩展已签名，按路径 LOAD 无需开启 allow_unsigned_extensions。
        let load_sql = match offline_extension_path(ext_file) {
            Some(p) if p.exists() => {
                let path_sql =
                    p.to_string_lossy().replace('\\', "/").replace('\'', "''");
                info!(extension = ext_file, path = %path_sql, "从本地文件加载扩展(离线)");
                format!("LOAD '{path_sql}';")
            }
            _ => {
                info!(extension = ext_file, "在线安装并加载扩展");
                format!("INSTALL {ext_short}; LOAD {ext_short};")
            }
        };
        conn.execute_batch(&load_sql)
            .with_context(|| format!("加载{ext_file}扩展失败"))
            .inspect_err(|e| {
                tracing::error!(error = %err_chain(e), extension = ext_file, "加载扩展失败")
            })?;

        // Vastbase/openGauss 系对 PG 二进制 COPY 协议的实现不兼容（拉取时报
        // "Unable to read binary COPY data, invalid header"，ATTACH/建表正常），
        // 统一改走文本协议拉取。两个开关分别对应新旧版本扩展的写法；个别版本
        // 可能不认识其中某个，失败仅告警不阻断。必须在 ATTACH 之前设置（协议
        // 模式随挂载固化）。
        if matches!(db_type, "postgres" | "postgresql") {
            for setting in [
                "SET pg_use_text_protocol = true",
                "SET pg_use_binary_copy = false",
            ] {
                if let Err(e) = conn.execute_batch(setting) {
                    tracing::warn!(
                        error = %e,
                        setting = %setting,
                        "设置postgres扩展兼容性参数失败(忽略)"
                    );
                }
            }
        }

        // 附加源数据库（若已附加则跳过）
        let db_exists = {
            let mut stmt = conn.prepare("SHOW DATABASES")?;
            let mut rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
            rows.any(|r| r.is_ok_and(|name| name.eq_ignore_ascii_case(db_name)))
        };
        if !db_exists {
            let attach_query = build_attach_query(
                db_type, db_host, db_port, db_name, db_user, db_pass,
            )?;
            info!(db_type = %db_type, db_name = %db_name, "正在附加数据库");
            conn.execute(&attach_query, []).context("附加数据库失败")?;
        }

        info!(db_type = %db_type, db_name = %db_name, "DuckDB 同步器初始化完成");
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_type: db_type.to_lowercase(),
            schema: db_schema.filter(|s| !s.is_empty()).map(str::to_string),
        })
    }

    /// 构造源表的三段式引用 `"<catalog>"."<schema>"."<table>"`。
    ///
    /// MySQL 的 DuckDB 扩展把源库挂成同名 schema（catalog 与 schema 同名），
    /// 直接用 db_name 即可；PostgreSQL 挂接后 catalog 内是真实的 PG schema，
    /// 使用数据源上显式配置的 schema，缺省 public。
    fn source_table_ref(&self, db_name: &str, table_name: &str) -> String {
        let schema = match self.db_type.as_str() {
            "postgres" | "postgresql" => {
                self.schema.as_deref().unwrap_or("public")
            }
            _ => db_name,
        };
        format!("\"{db_name}\".\"{schema}\".\"{table_name}\"")
    }

    /// 全量同步：将源表流式写入 DuckDB 原生表。
    ///
    /// 直接走 `INSERT INTO target SELECT * FROM <catalog>.<schema>.<table>`
    /// （schema 由 `source_table_ref` 按数据库类型解析），由
    /// DuckDB 的 mysql/postgres 扩展 scanner 在内部流式拉取并向量化写入。
    /// 单条 SQL 完成全表导入：无应用层分批、无 TEMP 表双写、无每批 round-trip。
    ///
    /// 取舍：
    /// - 全程无进度事件（仅在开始发一次 `Exporting`，由 `run_trans_task_with_progress`
    ///   在结束时发 `Done` 携带最终行数）
    /// - 不可中途取消（只能 kill 连接，目标表保持空）
    /// - 失败整体回滚（INSERT 在 DuckDB 隐式事务内）
    pub fn full_sync(
        &self,
        _source_code: &str,
        db_name: &str,
        table_name: &str,
        progress_tx: &Option<ProgressTx>,
    ) -> Result<usize> {
        validate_identifier(db_name)
            .context(format!("全量同步中无效的数据库名: {}", db_name))?;
        validate_identifier(table_name)
            .context(format!("全量同步中无效的表名: {}", table_name))?;

        emit(
            progress_tx,
            SyncProgressEvent::Exporting {
                phase: "full".to_string(),
            },
        );

        let source_ref = self.source_table_ref(db_name, table_name);

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("获取DuckDB连接锁失败: {}", e))?;

        // 一步重建目标表结构（CREATE OR REPLACE 替代 DROP+CREATE，少一次 round-trip）
        conn.execute(
            &format!(
                "CREATE OR REPLACE TABLE \"{}\" AS SELECT * FROM {} WHERE 1=0",
                table_name, source_ref
            ),
            [],
        )
        .inspect_err(|e| {
            tracing::error!(
                error = %err_chain(e),
                table_name = %table_name,
                "重建目标表结构失败"
            );
        })
        .context(format!("重建目标表 '{}' 失败", table_name))?;

        // 单条 INSERT 完成全表流式写入；execute() 返回 affected rows，无需额外 COUNT
        let rows_done = conn
            .execute(
                &format!(
                    "INSERT INTO \"{}\" SELECT * FROM {}",
                    table_name, source_ref
                ),
                [],
            )
            .inspect_err(|e| {
                tracing::error!(
                    error = %err_chain(e),
                    table_name = %table_name,
                    "流式写入目标表失败"
                );
            })
            .context(format!("流式写入表 '{}' 失败", table_name))?;

        info!(
            db_name = %db_name,
            table_name = %table_name,
            rows = rows_done,
            "成功全量同步到 DuckDB（流式模式）"
        );
        Ok(rows_done)
    }

    /// 增量同步（基于水位值）
    ///
    /// - `source_code`：数据源 code，用于 DuckDB schema 隔离（`src_{code}`）
    /// - `db_name`：源数据库名，用于 ATTACH 目录引用
    /// - 首次运行（watermark = None 或目标表不存在）等同全量同步，并捕获初始水位
    /// - auto_increment 策略：追加新行
    /// - timestamp 策略：按主键 DELETE + INSERT 实现 upsert
    #[allow(clippy::too_many_arguments)]
    pub fn incremental_sync(
        &self,
        source_code: &str,
        db_name: &str,
        table_name: &str,
        sync_strategy: &str,
        sync_column: &str,
        watermark: Option<&str>,
        primary_key: Option<&str>,
        progress_tx: &Option<ProgressTx>,
    ) -> Result<SyncResult> {
        // 检查 main schema 下目标表是否已存在
        let table_exists = {
            let conn = self
                .conn
                .lock()
                .map_err(|e| anyhow::anyhow!("获取DuckDB连接锁失败: {}", e))?;
            let check_sql = format!(
                "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'main' AND table_name = '{}'",
                table_name
            );
            conn.prepare(&check_sql)
                .and_then(|mut s| s.query_row([], |r| r.get::<_, i64>(0)))
                .unwrap_or(0)
                > 0
        };

        // 首次运行或表不存在：全量同步 + 捕获水位
        if watermark.is_none() || !table_exists {
            info!(table_name = %table_name, "首次增量同步，执行全量同步");
            let row_count =
                self.full_sync(source_code, db_name, table_name, progress_tx)?;
            let new_watermark = self.capture_watermark(
                &self.source_table_ref(db_name, table_name),
                sync_column,
            )?;
            return Ok(SyncResult {
                row_count,
                new_watermark: Some(new_watermark),
            });
        }

        let watermark_val = watermark.ok_or_else(|| {
            anyhow::anyhow!("增量同步缺少水位值: table={}", table_name)
        })?;
        validate_identifier(db_name)
            .context(format!("增量同步中无效的数据库名: {}", db_name))?;
        validate_identifier(table_name)
            .context(format!("增量同步中无效的表名: {}", table_name))?;
        validate_identifier(sync_column)
            .context(format!("增量同步中无效的同步列: {}", sync_column))?;

        info!(
            table_name = %table_name,
            sync_strategy = %sync_strategy,
            sync_column = %sync_column,
            watermark = %watermark_val,
            "开始增量同步"
        );

        emit(
            progress_tx,
            SyncProgressEvent::Exporting {
                phase: "delta".to_string(),
            },
        );

        let source_ref = self.source_table_ref(db_name, table_name);

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("获取DuckDB连接锁失败: {}", e))?;

        let delta_rows = if sync_strategy == "auto_increment" {
            let insert_sql = format!(
                "INSERT INTO \"{}\" SELECT * FROM {} WHERE \"{}\" > '{}'",
                table_name,
                source_ref,
                sync_column,
                escape_sql_string(watermark_val)
            );
            conn.execute(&insert_sql, []).context(format!(
                "auto_increment 增量插入表 '{}' 失败",
                table_name
            ))?
        } else {
            let pk = primary_key.ok_or_else(|| {
                anyhow::anyhow!(
                    "timestamp 策略需要指定 sync_primary_key 用于去重"
                )
            })?;
            validate_identifier(pk).context(format!(
                "incremental_sync 中无效的 primary_key: {}",
                pk
            ))?;

            let delete_sql = format!(
                "DELETE FROM \"{}\" WHERE \"{}\" IN (SELECT \"{}\" FROM {} WHERE \"{}\" >= '{}')",
                table_name,
                pk,
                pk,
                source_ref,
                sync_column,
                escape_sql_string(watermark_val)
            );
            conn.execute(&delete_sql, []).context(format!(
                "timestamp upsert DELETE 表 '{}' 失败",
                table_name
            ))?;

            let insert_sql = format!(
                "INSERT INTO \"{}\" SELECT * FROM {} WHERE \"{}\" >= '{}'",
                table_name,
                source_ref,
                sync_column,
                escape_sql_string(watermark_val)
            );
            conn.execute(&insert_sql, []).context(format!(
                "timestamp upsert INSERT 表 '{}' 失败",
                table_name
            ))?
        };

        drop(conn);

        let new_watermark = self.capture_watermark(&source_ref, sync_column)?;

        // 增量同步无 TotalRows 事件,但前端会在 progress 后通过 Done 事件
        // 收到最终 row_count;过程中 totalRows 保持 null,显示滚动条。
        emit(
            progress_tx,
            SyncProgressEvent::Progress {
                rows_done: delta_rows as u64,
            },
        );

        info!(
            table_name = %table_name,
            delta_rows = delta_rows,
            new_watermark = %new_watermark,
            "增量同步完成"
        );

        Ok(SyncResult {
            row_count: delta_rows,
            new_watermark: Some(new_watermark),
        })
    }

    /// 从源库捕获当前最大水位值。`source_ref` 为已解析的三段式源表引用。
    fn capture_watermark(
        &self,
        source_ref: &str,
        sync_column: &str,
    ) -> Result<String> {
        validate_identifier(sync_column)
            .context(format!("捕获水位中无效的同步列: {}", sync_column))?;

        let query = format!(
            "SELECT CAST(MAX(\"{}\") AS VARCHAR) FROM {}",
            sync_column, source_ref
        );

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("获取DuckDB连接锁失败: {}", e))?;

        let mut stmt = conn.prepare(&query).context("准备水位查询失败")?;
        let watermark: String = stmt
            .query_row([], |row| row.get(0))
            .context("查询水位值失败")?;

        Ok(watermark)
    }
}

/// 安装目录（exe 同目录）。
///
/// 优先取 `HDATA_HOME` 环境变量：Linux 的自带 glibc 包用包内 ld.so 启动
/// (`ld-linux.so --library-path … hdata-analysis.bin`)，这种启动方式下内核
/// 记录的 `/proc/self/exe` 是**加载器**而不是本程序，`current_exe()` 会拿到
/// `<prefix>/glibc/ld-linux-x86-64.so.2`，往上取父目录就跑偏了。启动器脚本
/// 会导出 `HDATA_HOME=<prefix>`。普通包没有这个变量，回退 `current_exe()`。
fn app_dir() -> Option<std::path::PathBuf> {
    if let Some(home) = std::env::var_os("HDATA_HOME")
        && !home.is_empty()
    {
        return Some(std::path::PathBuf::from(home));
    }
    Some(std::env::current_exe().ok()?.parent()?.to_path_buf())
}

/// 离线扩展文件路径：安装目录下 `extensions/<ext>.duckdb_extension`。
///
/// 该文件的版本必须与运行时 libduckdb 版本一致（当前 1.5.3），
/// 否则 LOAD 会因版本/平台不符而失败。开发环境下该文件不存在，
/// 调用方会自动回退到在线 INSTALL。
fn offline_extension_path(ext: &str) -> Option<std::path::PathBuf> {
    Some(
        app_dir()?
            .join("extensions")
            .join(format!("{ext}.duckdb_extension")),
    )
}

/// 根据数据库类型生成 DuckDB ATTACH 语句
///
/// MySQL:      `ATTACH '... database=D' AS D (TYPE mysql)`
/// PostgreSQL: `ATTACH '... dbname=D'   AS D (TYPE postgres)`
fn build_attach_query(
    db_type: &str,
    db_host: &str,
    db_port: u16,
    db_name: &str,
    db_user: &str,
    db_pass: &str,
) -> Result<String> {
    validate_identifier(db_name)
        .context(format!("构建附加查询中无效的数据库名: {}", db_name))?;

    match db_type {
        "mysql" => Ok(format!(
            "ATTACH 'host={} port={} user={} password={} database={}' AS \"{}\" (TYPE mysql)",
            escape_sql_string(db_host),
            db_port,
            escape_sql_string(db_user),
            escape_sql_string(db_pass),
            db_name,
            db_name
        )),
        "postgres" | "postgresql" => Ok(format!(
            "ATTACH 'host={} port={} user={} password={} dbname={}' AS \"{}\" (TYPE postgres)",
            escape_sql_string(db_host),
            db_port,
            escape_sql_string(db_user),
            escape_sql_string(db_pass),
            db_name,
            db_name
        )),
        other => Err(anyhow::anyhow!("不支持的数据库类型: {}", other)),
    }
}

/// Convert a DuckDB `ValueRef` into a `serde_json::Value`.
///
/// Handles all common DuckDB column types: null, bool, integers (signed &
/// unsigned), float/double, decimal, text, timestamp, date, and time.
/// Unknown variants fall back to a debug-string representation.
pub fn value_ref_to_json(val: duckdb::types::ValueRef) -> Value {
    match val {
        duckdb::types::ValueRef::Null => Value::Null,
        duckdb::types::ValueRef::Boolean(b) => Value::Bool(b),
        duckdb::types::ValueRef::TinyInt(i) => {
            Value::Number(serde_json::Number::from(i))
        }
        duckdb::types::ValueRef::SmallInt(i) => {
            Value::Number(serde_json::Number::from(i))
        }
        duckdb::types::ValueRef::Int(i) => {
            Value::Number(serde_json::Number::from(i))
        }
        duckdb::types::ValueRef::BigInt(i) => {
            Value::Number(serde_json::Number::from(i))
        }
        duckdb::types::ValueRef::HugeInt(i) => {
            Value::Number(serde_json::Number::from(i as i64))
        }
        duckdb::types::ValueRef::UTinyInt(i) => {
            Value::Number(serde_json::Number::from(i))
        }
        duckdb::types::ValueRef::USmallInt(i) => {
            Value::Number(serde_json::Number::from(i))
        }
        duckdb::types::ValueRef::UInt(i) => {
            Value::Number(serde_json::Number::from(i))
        }
        duckdb::types::ValueRef::UBigInt(i) => {
            Value::Number(serde_json::Number::from(i))
        }
        duckdb::types::ValueRef::Float(f) => {
            serde_json::Number::from_f64(f as f64)
                .map(Value::Number)
                .unwrap_or(Value::Null)
        }
        duckdb::types::ValueRef::Double(f) => serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        duckdb::types::ValueRef::Decimal(d) => {
            let s = d.normalize().to_string();
            serde_json::Number::from_str(&s)
                .map(Value::Number)
                .unwrap_or_else(|_| Value::String(s))
        }
        duckdb::types::ValueRef::Text(t) => {
            Value::String(String::from_utf8_lossy(t).to_string())
        }
        duckdb::types::ValueRef::Timestamp(unit, val) => {
            let micros: i64 = match unit {
                duckdb::types::TimeUnit::Second => val * 1_000_000,
                duckdb::types::TimeUnit::Millisecond => val * 1_000,
                duckdb::types::TimeUnit::Microsecond => val,
                duckdb::types::TimeUnit::Nanosecond => val / 1_000,
            };
            let secs = micros / 1_000_000;
            let nanos = ((micros % 1_000_000) * 1_000) as u32;
            match chrono::Utc.timestamp_opt(secs, nanos) {
                chrono::LocalResult::Single(dt) => {
                    Value::String(dt.format("%Y-%m-%d %H:%M:%S").to_string())
                }
                _ => Value::String(val.to_string()),
            }
        }
        duckdb::types::ValueRef::Date32(days) => {
            match chrono::Utc.timestamp_opt(days as i64 * 86400, 0) {
                chrono::LocalResult::Single(dt) => {
                    Value::String(dt.format("%Y-%m-%d").to_string())
                }
                _ => Value::String(days.to_string()),
            }
        }
        duckdb::types::ValueRef::Time64(unit, val) => {
            let micros: i64 = match unit {
                duckdb::types::TimeUnit::Second => val * 1_000_000,
                duckdb::types::TimeUnit::Millisecond => val * 1_000,
                duckdb::types::TimeUnit::Microsecond => val,
                duckdb::types::TimeUnit::Nanosecond => val / 1_000,
            };
            let h = micros / 3_600_000_000;
            let m = (micros % 3_600_000_000) / 60_000_000;
            let s = (micros % 60_000_000) / 1_000_000;
            Value::String(format!("{:02}:{:02}:{:02}", h, m, s))
        }
        other => Value::String(format!("{other:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory_syncer(db_type: &str, schema: Option<&str>) -> DbToDuckdbSync {
        DbToDuckdbSync {
            conn: Arc::new(Mutex::new(Connection::open_in_memory().unwrap())),
            db_type: db_type.to_string(),
            schema: schema.map(str::to_string),
        }
    }

    /// MySQL 的 DuckDB 扩展把源库挂成同名 schema，源表引用为
    /// `"<db>"."<db>"."<table>"`。
    #[test]
    fn mysql_source_ref_uses_db_name_as_schema() {
        let syncer = memory_syncer("mysql", None);
        assert_eq!(
            syncer.source_table_ref("his_db", "t1"),
            "\"his_db\".\"his_db\".\"t1\""
        );
    }

    /// PG 使用数据源上显式配置的 schema，不做任何猜测。
    #[test]
    fn postgres_source_ref_uses_configured_schema() {
        let syncer = memory_syncer("postgresql", Some("wisdom_medicine"));
        assert_eq!(
            syncer.source_table_ref("pg_db", "t1"),
            "\"pg_db\".\"wisdom_medicine\".\"t1\""
        );
    }

    /// PG 未配置 schema 时缺省回退 public。
    #[test]
    fn postgres_source_ref_defaults_to_public() {
        let syncer = memory_syncer("postgres", None);
        assert_eq!(
            syncer.source_table_ref("pg_db", "t1"),
            "\"pg_db\".\"public\".\"t1\""
        );
    }

    /// 自带 glibc 包用包内 ld.so 启动时 `current_exe()` 指向加载器，
    /// 必须靠 `HDATA_HOME` 才能找到 `extensions/`。
    ///
    /// HDATA_HOME 是进程级状态，两个分支放同一个测试里顺序跑，避免并行串扰。
    #[test]
    fn app_dir_prefers_hdata_home_then_falls_back() {
        unsafe { std::env::set_var("HDATA_HOME", "/opt/hdata-analysis") };
        assert_eq!(
            offline_extension_path("mysql_scanner"),
            Some(std::path::PathBuf::from(
                "/opt/hdata-analysis/extensions/mysql_scanner.duckdb_extension"
            ))
        );

        // 空值视同没设，回退 current_exe 的父目录
        let exe_dir = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        unsafe { std::env::set_var("HDATA_HOME", "") };
        assert_eq!(app_dir(), Some(exe_dir.clone()));

        unsafe { std::env::remove_var("HDATA_HOME") };
        assert_eq!(app_dir(), Some(exe_dir));

        #[cfg(unix)]
        loads_offline_extension_from_hdata_home();
    }

    /// 回归：`HDATA_HOME/extensions/` 下的官方扩展必须能直接 LOAD 成功，
    /// 不能退化成联网 INSTALL（内网无外网，退化即失败）。
    ///
    /// 只跑到 LOAD：ATTACH 指向一个必然拒绝连接的地址，报错停在“附加数据库失败”
    /// 就说明扩展已经加载好了。依赖 vendor 里的官方扩展文件（git 不跟踪），
    /// 缺了就跳过。
    #[cfg(unix)] // 依赖 unix symlink 与 Linux 版扩展二进制
    fn loads_offline_extension_from_hdata_home() {
        let vendored = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../scripts/build/vendor/duckdb-ext-1.5.3-linux_amd64")
            .join("mysql_scanner.duckdb_extension");
        if !vendored.exists() {
            eprintln!("跳过离线 LOAD 回归：缺 {}", vendored.display());
            return;
        }

        let home = std::env::temp_dir()
            .join(format!("hdata-offline-ext-{}", std::process::id()));
        let ext_dir = home.join("extensions");
        std::fs::create_dir_all(&ext_dir).unwrap();
        let linked = ext_dir.join("mysql_scanner.duckdb_extension");
        let _ = std::fs::remove_file(&linked);
        std::os::unix::fs::symlink(
            std::fs::canonicalize(&vendored).unwrap(),
            &linked,
        )
        .unwrap();

        unsafe { std::env::set_var("HDATA_HOME", &home) };
        let result = DbToDuckdbSync::new(
            "mysql",
            "127.0.0.1",
            1, // 必然 ECONNREFUSED
            "nonexistent_db",
            "u",
            "p",
            None,
            &home.join("probe.duckdb"),
        );
        let err = match result {
            Ok(_) => panic!("连不上 MySQL，不该成功"),
            Err(e) => e,
        };
        unsafe { std::env::remove_var("HDATA_HOME") };
        std::fs::remove_dir_all(&home).ok();

        let msg = format!("{err:#}");
        assert!(
            !msg.contains("加载mysql_scanner扩展失败"),
            "扩展没能从 HDATA_HOME 离线加载: {msg}"
        );
        assert!(msg.contains("附加数据库失败"), "预期停在 ATTACH: {msg}");
    }
}
