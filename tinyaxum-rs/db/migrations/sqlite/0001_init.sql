-- =============================================================
-- hdata-analysis 初始化脚本 (SQLite 兼容版)
-- =============================================================

PRAGMA foreign_keys = ON;

-- 用户表
CREATE TABLE IF NOT EXISTS sys_user (
    user_id         INTEGER PRIMARY KEY AUTOINCREMENT,
    username        VARCHAR(100)  NOT NULL UNIQUE,
    password        VARCHAR(255)  NOT NULL,
    nick_name       VARCHAR(100)  NOT NULL,
    email           VARCHAR(255),
    active          VARCHAR(10)   NOT NULL DEFAULT 'Y',
    version         INTEGER       NOT NULL DEFAULT 1,
    created_by      INTEGER       NOT NULL,
    created_date    TEXT          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by   INTEGER    NOT NULL,
    last_modified_date TEXT       NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by      INTEGER,
    deleted_date    TEXT
);

-- 数据源表
CREATE TABLE IF NOT EXISTS biz_data_source (
    data_source_id  INTEGER PRIMARY KEY AUTOINCREMENT,
    code            VARCHAR(100)  NOT NULL UNIQUE,
    name            VARCHAR(255)  NOT NULL,
    remark          TEXT,
    db_type         VARCHAR(50)   NOT NULL,
    db_host         VARCHAR(255)  NOT NULL,
    db_port         INTEGER       NOT NULL,
    db_name         VARCHAR(100)  NOT NULL,
    db_schema       VARCHAR(128),
    db_username     VARCHAR(100)  NOT NULL,
    db_password     VARCHAR(500)  NOT NULL,
    version         INTEGER       NOT NULL DEFAULT 1,
    created_by      INTEGER       NOT NULL,
    created_date    TEXT          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by   INTEGER    NOT NULL,
    last_modified_date TEXT       NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by      INTEGER,
    deleted_date    TEXT
);

-- 数据传输任务表
CREATE TABLE IF NOT EXISTS biz_trans_task (
    trans_task_id   INTEGER PRIMARY KEY AUTOINCREMENT,
    data_source_id  INTEGER       NOT NULL REFERENCES biz_data_source(data_source_id),
    table_name      VARCHAR(255)  NOT NULL,
    table_comment   VARCHAR(500)  NOT NULL,
    remark          TEXT,
    row_count       INTEGER       NOT NULL DEFAULT 0,
    last_trans_time TEXT          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    task_cron       VARCHAR(100)  NOT NULL,
    task_status     VARCHAR(20)   NOT NULL DEFAULT 'STOPPED',
    task_uuid       VARCHAR(100),
    sync_mode       TEXT          NOT NULL DEFAULT 'full',
    sync_strategy   TEXT          DEFAULT NULL,
    sync_column     TEXT          DEFAULT NULL,
    sync_watermark  TEXT          DEFAULT NULL,
    sync_primary_key TEXT         DEFAULT NULL,
    is_syncing      INTEGER       NOT NULL DEFAULT 0,
    version         INTEGER       NOT NULL DEFAULT 1,
    created_by      INTEGER       NOT NULL,
    created_date    TEXT          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by   INTEGER    NOT NULL,
    last_modified_date TEXT       NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by      INTEGER,
    deleted_date    TEXT
);

-- SQL 片段表
CREATE TABLE IF NOT EXISTS biz_sql_snippet (
    sql_snippet_id      INTEGER PRIMARY KEY AUTOINCREMENT,
    sql_snippet_name    VARCHAR(255) NOT NULL,
    sql_snippet_content TEXT         NOT NULL,
    sql_snippet_desc    TEXT,
    sql_snippet_tag     VARCHAR(500),
    version             INTEGER       NOT NULL DEFAULT 1,
    created_by          INTEGER       NOT NULL,
    created_date        TEXT          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    INTEGER       NOT NULL,
    last_modified_date  TEXT          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by          INTEGER,
    deleted_date        TEXT
);

-- SQL 查询表
CREATE TABLE IF NOT EXISTS biz_sql_query (
    sql_query_id        INTEGER PRIMARY KEY AUTOINCREMENT,
    sql_query_name      VARCHAR(255) NOT NULL,
    sql_query_desc      TEXT,
    sql_query_content   TEXT         NOT NULL,
    data_source_id      INTEGER       REFERENCES biz_data_source(data_source_id),
    result_meta_updated_at TEXT,
    is_collection       INTEGER       NOT NULL DEFAULT 1,
    viz_config          TEXT          NULL,
    module              VARCHAR(100),
    category            VARCHAR(100),
    version             INTEGER       NOT NULL DEFAULT 1,
    created_by          INTEGER       NOT NULL,
    created_date        TEXT          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    INTEGER       NOT NULL,
    last_modified_date  TEXT          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by          INTEGER,
    deleted_date        TEXT
);

-- 数据源表元数据
CREATE TABLE IF NOT EXISTS biz_data_source_table_meta (
    table_meta_id       INTEGER PRIMARY KEY AUTOINCREMENT,
    data_source_id      INTEGER       NOT NULL REFERENCES biz_data_source(data_source_id),
    table_name          VARCHAR(255)  NOT NULL,
    comment_label       VARCHAR(500),
    table_desc          TEXT,
    version             INTEGER       NOT NULL DEFAULT 1,
    created_by          INTEGER       NOT NULL,
    created_date        TEXT          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    INTEGER       NOT NULL,
    last_modified_date  TEXT          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by          INTEGER,
    deleted_date        TEXT
);

CREATE INDEX IF NOT EXISTS idx_table_meta_ds ON biz_data_source_table_meta(data_source_id);

-- 数据源列元数据
CREATE TABLE IF NOT EXISTS biz_data_source_column_meta (
    column_meta_id      INTEGER PRIMARY KEY AUTOINCREMENT,
    table_meta_id       INTEGER       NOT NULL REFERENCES biz_data_source_table_meta(table_meta_id),
    column_name         VARCHAR(255)  NOT NULL,
    comment_label       VARCHAR(500),
    field_type          VARCHAR(32)   NOT NULL DEFAULT 'TEXT',
    format_pattern      VARCHAR(100),
    field_desc          TEXT,
    source_physical_type VARCHAR(200),
    ordinal_position    INTEGER       NOT NULL DEFAULT 0,
    is_primary_key      INTEGER       NOT NULL DEFAULT 0,
    version             INTEGER       NOT NULL DEFAULT 1,
    created_by          INTEGER       NOT NULL,
    created_date        TEXT          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    INTEGER       NOT NULL,
    last_modified_date  TEXT          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by          INTEGER,
    deleted_date        TEXT
);

CREATE INDEX IF NOT EXISTS idx_column_meta_table ON biz_data_source_column_meta(table_meta_id);

-- SQL 查询结果字段
CREATE TABLE IF NOT EXISTS biz_sql_query_result_field (
    result_field_id     INTEGER PRIMARY KEY AUTOINCREMENT,
    sql_query_id        INTEGER       NOT NULL REFERENCES biz_sql_query(sql_query_id),
    field_name          VARCHAR(255)  NOT NULL,
    comment_label       VARCHAR(500),
    field_type          VARCHAR(32)   NOT NULL DEFAULT 'TEXT',
    format_pattern      VARCHAR(100),
    field_desc          TEXT,
    version             INTEGER       NOT NULL DEFAULT 1,
    created_by          INTEGER       NOT NULL,
    created_date        TEXT          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    INTEGER       NOT NULL,
    last_modified_date  TEXT          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by          INTEGER,
    deleted_date        TEXT
);

CREATE INDEX IF NOT EXISTS idx_result_field_query ON biz_sql_query_result_field(sql_query_id);

-- SQL 查询参数字段
CREATE TABLE IF NOT EXISTS biz_sql_query_param_field (
    param_field_id      INTEGER PRIMARY KEY AUTOINCREMENT,
    sql_query_id        INTEGER       NOT NULL REFERENCES biz_sql_query(sql_query_id),
    field_name          VARCHAR(255)  NOT NULL,
    comment_label       VARCHAR(500),
    field_type          VARCHAR(32)   NOT NULL DEFAULT 'TEXT',
    format_pattern      VARCHAR(100),
    field_desc          TEXT,
    required_flag       INTEGER       NOT NULL DEFAULT 0,
    version             INTEGER       NOT NULL DEFAULT 1,
    created_by          INTEGER       NOT NULL,
    created_date        TEXT          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    INTEGER       NOT NULL,
    last_modified_date  TEXT          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by          INTEGER,
    deleted_date        TEXT
);

CREATE INDEX IF NOT EXISTS idx_param_field_query ON biz_sql_query_param_field(sql_query_id);

-- 看板配置（按用户）
CREATE TABLE IF NOT EXISTS sys_user_dashboard_config (
    user_id            INTEGER      NOT NULL PRIMARY KEY REFERENCES sys_user(user_id),
    config_json        TEXT         NULL,
    version            INTEGER      NOT NULL DEFAULT 1,
    created_by         INTEGER      NOT NULL,
    created_date       TEXT         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by   INTEGER      NOT NULL,
    last_modified_date TEXT         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by         INTEGER,
    deleted_date       TEXT
);

-- HTTP API 表
CREATE TABLE IF NOT EXISTS biz_http_api (
    http_api_id         INTEGER PRIMARY KEY AUTOINCREMENT,
    http_api_name       VARCHAR(255) NOT NULL,
    http_api_path       VARCHAR(255) NOT NULL,
    http_api_desc       TEXT,
    response_template   TEXT,
    module              VARCHAR(100),
    category            VARCHAR(100),
    enabled             INTEGER      NOT NULL DEFAULT 1,
    default_params      TEXT         NULL,
    version             INTEGER      NOT NULL DEFAULT 1,
    created_by          INTEGER      NOT NULL,
    created_date        TEXT         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    INTEGER      NOT NULL,
    last_modified_date  TEXT         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by          INTEGER,
    deleted_date        TEXT         NULL,
    UNIQUE (http_api_path, deleted_date)
);

-- HTTP API 与查询关联表
CREATE TABLE IF NOT EXISTS biz_http_api_query (
    http_api_query_id   INTEGER PRIMARY KEY AUTOINCREMENT,
    http_api_id         INTEGER      NOT NULL REFERENCES biz_http_api(http_api_id),
    sql_query_id        INTEGER      NOT NULL REFERENCES biz_sql_query(sql_query_id),
    query_alias         VARCHAR(255) NOT NULL,
    sort_order          INTEGER      NOT NULL DEFAULT 0,
    version             INTEGER      NOT NULL DEFAULT 1,
    created_by          INTEGER      NOT NULL,
    created_date        TEXT         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    INTEGER      NOT NULL,
    last_modified_date  TEXT         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by          INTEGER,
    deleted_date        TEXT         NULL
);

CREATE INDEX IF NOT EXISTS idx_http_api_query_api ON biz_http_api_query(http_api_id);
CREATE INDEX IF NOT EXISTS idx_http_api_query_sql ON biz_http_api_query(sql_query_id);

-- AI Provider 配置表
CREATE TABLE IF NOT EXISTS biz_ai_provider (
    ai_provider_id      INTEGER PRIMARY KEY AUTOINCREMENT,
    name                TEXT     NOT NULL,
    base_url            TEXT     NOT NULL,
    api_key             TEXT     NOT NULL,
    model               TEXT     NOT NULL,
    system_prompt       TEXT     NOT NULL,
    temperature         REAL     NOT NULL DEFAULT 0.30,
    is_default          INTEGER  NOT NULL DEFAULT 0,
    version             INTEGER  NOT NULL DEFAULT 1,
    created_by          INTEGER  NOT NULL,
    created_date        TEXT     NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    last_modified_by    INTEGER  NOT NULL,
    last_modified_date  TEXT     NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    deleted_by          INTEGER,
    deleted_date        TEXT
);

-- AI 会话表
CREATE TABLE IF NOT EXISTS biz_ai_conversation (
    ai_conversation_id  INTEGER PRIMARY KEY AUTOINCREMENT,
    sql_query_id        INTEGER NOT NULL,
    ai_provider_id      INTEGER NOT NULL,
    title               TEXT    NOT NULL DEFAULT '新会话',
    version             INTEGER NOT NULL DEFAULT 1,
    created_by          INTEGER NOT NULL,
    created_date        TEXT    NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    last_modified_by    INTEGER NOT NULL,
    last_modified_date  TEXT    NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    deleted_by          INTEGER,
    deleted_date        TEXT
);
CREATE INDEX IF NOT EXISTS idx_ai_conv_sql_query ON biz_ai_conversation(sql_query_id);
CREATE INDEX IF NOT EXISTS idx_ai_conv_provider ON biz_ai_conversation(ai_provider_id);

-- AI 消息表
CREATE TABLE IF NOT EXISTS biz_ai_message (
    ai_message_id       INTEGER PRIMARY KEY AUTOINCREMENT,
    ai_conversation_id  INTEGER NOT NULL,
    role                TEXT    NOT NULL,
    content             TEXT    NOT NULL,
    attached_schema     INTEGER NOT NULL DEFAULT 0,
    prompt_tokens       INTEGER,
    completion_tokens   INTEGER,
    version             INTEGER NOT NULL DEFAULT 1,
    created_by          INTEGER NOT NULL,
    created_date        TEXT    NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    last_modified_by    INTEGER NOT NULL,
    last_modified_date  TEXT    NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    deleted_by          INTEGER,
    deleted_date        TEXT
);
CREATE INDEX IF NOT EXISTS idx_ai_msg_conv ON biz_ai_message(ai_conversation_id);
