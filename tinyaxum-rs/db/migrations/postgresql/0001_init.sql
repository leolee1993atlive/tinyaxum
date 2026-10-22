-- =============================================================
-- hdata-analysis 初始化脚本 (PostgreSQL 版本)
-- =============================================================

-- 用户表
CREATE TABLE IF NOT EXISTS sys_user (
    user_id         BIGSERIAL PRIMARY KEY,
    username        VARCHAR(100)  NOT NULL UNIQUE,
    password        VARCHAR(255)  NOT NULL,
    nick_name       VARCHAR(100)  NOT NULL,
    email           VARCHAR(255),
    active          VARCHAR(10)   NOT NULL DEFAULT 'Y',
    version         BIGINT        NOT NULL DEFAULT 1,
    created_by      BIGINT        NOT NULL,
    created_date    TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    last_modified_by   BIGINT     NOT NULL,
    last_modified_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_by      BIGINT,
    deleted_date    TIMESTAMPTZ
);

-- 数据源表
CREATE TABLE IF NOT EXISTS biz_data_source (
    data_source_id  BIGSERIAL PRIMARY KEY,
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
    version         BIGINT        NOT NULL DEFAULT 1,
    created_by      BIGINT        NOT NULL,
    created_date    TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    last_modified_by   BIGINT     NOT NULL,
    last_modified_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_by      BIGINT,
    deleted_date    TIMESTAMPTZ
);

-- 数据传输任务表
CREATE TABLE IF NOT EXISTS biz_trans_task (
    trans_task_id   BIGSERIAL PRIMARY KEY,
    data_source_id  BIGINT        NOT NULL REFERENCES biz_data_source(data_source_id),
    table_name      VARCHAR(255)  NOT NULL,
    table_comment   VARCHAR(500)  NOT NULL,
    remark          TEXT,
    row_count       BIGINT        NOT NULL DEFAULT 0,
    last_trans_time TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    task_cron       VARCHAR(100)  NOT NULL,
    task_status     VARCHAR(20)   NOT NULL DEFAULT 'STOPPED',
    task_uuid       VARCHAR(100),
    sync_mode        VARCHAR(20)  NOT NULL DEFAULT 'full',
    sync_strategy    VARCHAR(20)  DEFAULT NULL,
    sync_column      VARCHAR(100) DEFAULT NULL,
    sync_watermark   TEXT         DEFAULT NULL,
    sync_primary_key VARCHAR(100) DEFAULT NULL,
    is_syncing      INTEGER       NOT NULL DEFAULT 0,
    version         BIGINT        NOT NULL DEFAULT 1,
    created_by      BIGINT        NOT NULL,
    created_date    TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    last_modified_by   BIGINT     NOT NULL,
    last_modified_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_by      BIGINT,
    deleted_date    TIMESTAMPTZ
);

-- SQL 片段表
CREATE TABLE IF NOT EXISTS biz_sql_snippet (
    sql_snippet_id      BIGSERIAL PRIMARY KEY,
    sql_snippet_name    VARCHAR(255) NOT NULL,
    sql_snippet_content TEXT         NOT NULL,
    sql_snippet_desc    TEXT,
    sql_snippet_tag     VARCHAR(500),
    version             BIGINT       NOT NULL DEFAULT 1,
    created_by          BIGINT       NOT NULL,
    created_date        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    last_modified_by    BIGINT       NOT NULL,
    last_modified_date  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_by          BIGINT,
    deleted_date        TIMESTAMPTZ
);

-- SQL 查询表
CREATE TABLE IF NOT EXISTS biz_sql_query (
    sql_query_id        BIGSERIAL PRIMARY KEY,
    sql_query_name      VARCHAR(255) NOT NULL,
    sql_query_desc      TEXT,
    sql_query_content   TEXT         NOT NULL,
    data_source_id      BIGINT       REFERENCES biz_data_source(data_source_id),
    result_meta_updated_at TIMESTAMPTZ,
    is_collection       "char"       NOT NULL DEFAULT 1::int::"char",
    viz_config          JSONB        NULL,
    module              VARCHAR(100),
    category            VARCHAR(100),
    version             BIGINT       NOT NULL DEFAULT 1,
    created_by          BIGINT       NOT NULL,
    created_date        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    last_modified_by    BIGINT       NOT NULL,
    last_modified_date  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_by          BIGINT,
    deleted_date        TIMESTAMPTZ
);

-- 数据源表元数据
CREATE TABLE IF NOT EXISTS biz_data_source_table_meta (
    table_meta_id       BIGSERIAL PRIMARY KEY,
    data_source_id      BIGINT        NOT NULL REFERENCES biz_data_source(data_source_id),
    table_name          VARCHAR(255)  NOT NULL,
    comment_label       VARCHAR(500),
    table_desc          TEXT,
    version             BIGINT        NOT NULL DEFAULT 1,
    created_by          BIGINT        NOT NULL,
    created_date        TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    last_modified_by    BIGINT        NOT NULL,
    last_modified_date  TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    deleted_by          BIGINT,
    deleted_date        TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_table_meta_ds ON biz_data_source_table_meta(data_source_id);

-- 数据源列元数据
CREATE TABLE IF NOT EXISTS biz_data_source_column_meta (
    column_meta_id      BIGSERIAL PRIMARY KEY,
    table_meta_id       BIGINT        NOT NULL REFERENCES biz_data_source_table_meta(table_meta_id),
    column_name         VARCHAR(255)  NOT NULL,
    comment_label       VARCHAR(500),
    field_type          VARCHAR(32)   NOT NULL DEFAULT 'TEXT',
    format_pattern      VARCHAR(100),
    field_desc          TEXT,
    source_physical_type VARCHAR(200),
    ordinal_position    INTEGER       NOT NULL DEFAULT 0,
    is_primary_key      INTEGER       NOT NULL DEFAULT 0,
    version             BIGINT        NOT NULL DEFAULT 1,
    created_by          BIGINT        NOT NULL,
    created_date        TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    last_modified_by    BIGINT        NOT NULL,
    last_modified_date  TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    deleted_by          BIGINT,
    deleted_date        TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_column_meta_table ON biz_data_source_column_meta(table_meta_id);

-- SQL 查询结果字段
CREATE TABLE IF NOT EXISTS biz_sql_query_result_field (
    result_field_id     BIGSERIAL PRIMARY KEY,
    sql_query_id        BIGINT        NOT NULL REFERENCES biz_sql_query(sql_query_id),
    field_name          VARCHAR(255)  NOT NULL,
    comment_label       VARCHAR(500),
    field_type          VARCHAR(32)   NOT NULL DEFAULT 'TEXT',
    format_pattern      VARCHAR(100),
    field_desc          TEXT,
    version             BIGINT        NOT NULL DEFAULT 1,
    created_by          BIGINT        NOT NULL,
    created_date        TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    last_modified_by    BIGINT        NOT NULL,
    last_modified_date  TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    deleted_by          BIGINT,
    deleted_date        TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_result_field_query ON biz_sql_query_result_field(sql_query_id);

-- SQL 查询参数字段
CREATE TABLE IF NOT EXISTS biz_sql_query_param_field (
    param_field_id      BIGSERIAL PRIMARY KEY,
    sql_query_id        BIGINT        NOT NULL REFERENCES biz_sql_query(sql_query_id),
    field_name          VARCHAR(255)  NOT NULL,
    comment_label       VARCHAR(500),
    field_type          VARCHAR(32)   NOT NULL DEFAULT 'TEXT',
    format_pattern      VARCHAR(100),
    field_desc          TEXT,
    required_flag       INTEGER      NOT NULL DEFAULT 0,
    version             BIGINT        NOT NULL DEFAULT 1,
    created_by          BIGINT        NOT NULL,
    created_date        TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    last_modified_by    BIGINT        NOT NULL,
    last_modified_date  TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    deleted_by          BIGINT,
    deleted_date        TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_param_field_query ON biz_sql_query_param_field(sql_query_id);

-- 看板配置（按用户）
CREATE TABLE IF NOT EXISTS sys_user_dashboard_config (
    user_id            BIGINT       NOT NULL PRIMARY KEY REFERENCES sys_user(user_id),
    config_json        JSONB        NULL,
    version            BIGINT       NOT NULL DEFAULT 1,
    created_by         BIGINT       NOT NULL,
    created_date       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    last_modified_by   BIGINT       NOT NULL,
    last_modified_date TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_by         BIGINT,
    deleted_date       TIMESTAMPTZ
);

-- HTTP API 表
CREATE TABLE IF NOT EXISTS biz_http_api (
    http_api_id         BIGSERIAL PRIMARY KEY,
    http_api_name       VARCHAR(255) NOT NULL,
    http_api_path       VARCHAR(255) NOT NULL,
    http_api_desc       TEXT,
    response_template   TEXT,
    module              VARCHAR(100),
    category            VARCHAR(100),
    enabled             INTEGER      NOT NULL DEFAULT 1,
    default_params      JSONB        NULL,
    version             BIGINT       NOT NULL DEFAULT 1,
    created_by          BIGINT       NOT NULL,
    created_date        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    last_modified_by    BIGINT       NOT NULL,
    last_modified_date  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_by          BIGINT,
    deleted_date        TIMESTAMPTZ  NULL,
    UNIQUE (http_api_path, deleted_date)
);

-- HTTP API 与查询关联表
CREATE TABLE IF NOT EXISTS biz_http_api_query (
    http_api_query_id   BIGSERIAL PRIMARY KEY,
    http_api_id         BIGINT       NOT NULL REFERENCES biz_http_api(http_api_id),
    sql_query_id        BIGINT       NOT NULL REFERENCES biz_sql_query(sql_query_id),
    query_alias         VARCHAR(255) NOT NULL,
    sort_order          BIGINT       NOT NULL DEFAULT 0,
    version             BIGINT       NOT NULL DEFAULT 1,
    created_by          BIGINT       NOT NULL,
    created_date        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    last_modified_by    BIGINT       NOT NULL,
    last_modified_date  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_by          BIGINT,
    deleted_date        TIMESTAMPTZ  NULL
);

CREATE INDEX IF NOT EXISTS idx_http_api_query_api ON biz_http_api_query(http_api_id);
CREATE INDEX IF NOT EXISTS idx_http_api_query_sql ON biz_http_api_query(sql_query_id);

-- AI Provider 配置表
CREATE TABLE IF NOT EXISTS biz_ai_provider (
    ai_provider_id      BIGSERIAL PRIMARY KEY,
    name                VARCHAR(64)  NOT NULL,
    base_url            VARCHAR(255) NOT NULL,
    api_key             VARCHAR(512) NOT NULL,
    model               VARCHAR(128) NOT NULL,
    system_prompt       TEXT         NOT NULL,
    temperature         NUMERIC(3,2) NOT NULL DEFAULT 0.30,
    is_default          "char"       NOT NULL DEFAULT 0::int::"char",
    version             BIGINT       NOT NULL DEFAULT 1,
    created_by          BIGINT       NOT NULL,
    created_date        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    last_modified_by    BIGINT       NOT NULL,
    last_modified_date  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_by          BIGINT,
    deleted_date        TIMESTAMPTZ
);

-- AI 会话表
CREATE TABLE IF NOT EXISTS biz_ai_conversation (
    ai_conversation_id  BIGSERIAL PRIMARY KEY,
    sql_query_id        BIGINT       NOT NULL,
    ai_provider_id      BIGINT       NOT NULL,
    title               VARCHAR(128) NOT NULL DEFAULT '新会话',
    version             BIGINT       NOT NULL DEFAULT 1,
    created_by          BIGINT       NOT NULL,
    created_date        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    last_modified_by    BIGINT       NOT NULL,
    last_modified_date  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_by          BIGINT,
    deleted_date        TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_ai_conv_sql_query ON biz_ai_conversation(sql_query_id);
CREATE INDEX IF NOT EXISTS idx_ai_conv_provider ON biz_ai_conversation(ai_provider_id);

-- AI 消息表
CREATE TABLE IF NOT EXISTS biz_ai_message (
    ai_message_id       BIGSERIAL PRIMARY KEY,
    ai_conversation_id  BIGINT       NOT NULL,
    role                VARCHAR(16)  NOT NULL,
    content             TEXT         NOT NULL,
    attached_schema     "char"       NOT NULL DEFAULT 0::int::"char",
    prompt_tokens       INTEGER,
    completion_tokens   INTEGER,
    version             BIGINT       NOT NULL DEFAULT 1,
    created_by          BIGINT       NOT NULL,
    created_date        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    last_modified_by    BIGINT       NOT NULL,
    last_modified_date  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_by          BIGINT,
    deleted_date        TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_ai_msg_conv ON biz_ai_message(ai_conversation_id);
