-- =============================================================
-- hdata-analysis 初始化脚本 (MySQL 版本)
-- =============================================================

-- 用户表
CREATE TABLE IF NOT EXISTS sys_user (
    user_id         BIGINT AUTO_INCREMENT PRIMARY KEY,
    username        VARCHAR(100)  NOT NULL UNIQUE,
    password        VARCHAR(255)  NOT NULL,
    nick_name       VARCHAR(100)  NOT NULL,
    email           VARCHAR(255),
    active          VARCHAR(10)   NOT NULL DEFAULT 'Y',
    version         BIGINT        NOT NULL DEFAULT 1,
    created_by      BIGINT        NOT NULL,
    created_date    TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by   BIGINT     NOT NULL,
    last_modified_date TIMESTAMP  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by      BIGINT,
    deleted_date    TIMESTAMP NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- 数据源表
CREATE TABLE IF NOT EXISTS biz_data_source (
    data_source_id  BIGINT AUTO_INCREMENT PRIMARY KEY,
    code            VARCHAR(100)  NOT NULL UNIQUE,
    name            VARCHAR(255)  NOT NULL,
    remark          TEXT,
    db_type         VARCHAR(50)   NOT NULL,
    db_host         VARCHAR(255)  NOT NULL,
    db_port         INT           NOT NULL,
    db_name         VARCHAR(100)  NOT NULL,
    db_schema       VARCHAR(128)  NULL COMMENT 'PG schema（仅 PostgreSQL 用，缺省 public）',
    db_username     VARCHAR(100)  NOT NULL,
    db_password     VARCHAR(500)  NOT NULL,
    version         BIGINT        NOT NULL DEFAULT 1,
    created_by      BIGINT        NOT NULL,
    created_date    TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by   BIGINT     NOT NULL,
    last_modified_date TIMESTAMP  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by      BIGINT,
    deleted_date    TIMESTAMP NULL,
    INDEX idx_data_source_code (code)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- 数据传输任务表
CREATE TABLE IF NOT EXISTS biz_trans_task (
    trans_task_id   BIGINT AUTO_INCREMENT PRIMARY KEY,
    data_source_id  BIGINT        NOT NULL,
    table_name      VARCHAR(255)  NOT NULL,
    table_comment   VARCHAR(500)  NOT NULL,
    remark          TEXT,
    row_count       BIGINT        NOT NULL DEFAULT 0,
    last_trans_time TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    task_cron       VARCHAR(100)  NOT NULL,
    task_status     VARCHAR(20)   NOT NULL DEFAULT 'STOPPED',
    task_uuid       VARCHAR(100),
    sync_mode        VARCHAR(20)  NOT NULL DEFAULT 'full' COMMENT '同步模式: full | incremental | realtime',
    sync_strategy    VARCHAR(20)  DEFAULT NULL COMMENT '增量策略: timestamp | auto_increment',
    sync_column      VARCHAR(100) DEFAULT NULL COMMENT '增量判断列名 (如 updated_at, id)',
    sync_watermark   TEXT         DEFAULT NULL COMMENT '上次同步水位值 (JSON)',
    sync_primary_key VARCHAR(100) DEFAULT NULL COMMENT '主键列名, 用于 upsert 去重',
    is_syncing      INT           NOT NULL DEFAULT 0 COMMENT '是否正在执行同步 (1=运行中, 0=空闲)',
    version         BIGINT        NOT NULL DEFAULT 1,
    created_by      BIGINT        NOT NULL,
    created_date    TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by   BIGINT     NOT NULL,
    last_modified_date TIMESTAMP  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by      BIGINT,
    deleted_date    TIMESTAMP NULL,
    CONSTRAINT fk_trans_task_data_source FOREIGN KEY (data_source_id)
        REFERENCES biz_data_source(data_source_id),
    INDEX idx_trans_task_source (data_source_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- SQL 片段表
CREATE TABLE IF NOT EXISTS biz_sql_snippet (
    sql_snippet_id      BIGINT AUTO_INCREMENT PRIMARY KEY,
    sql_snippet_name    VARCHAR(255) NOT NULL,
    sql_snippet_content TEXT         NOT NULL,
    sql_snippet_desc    TEXT,
    sql_snippet_tag     VARCHAR(500),
    version             BIGINT       NOT NULL DEFAULT 1,
    created_by          BIGINT       NOT NULL,
    created_date        TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    BIGINT       NOT NULL,
    last_modified_date  TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by          BIGINT,
    deleted_date        TIMESTAMP NULL,
    INDEX idx_sql_snippet_name (sql_snippet_name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- SQL 查询表
CREATE TABLE IF NOT EXISTS biz_sql_query (
    sql_query_id        BIGINT AUTO_INCREMENT PRIMARY KEY,
    sql_query_name      VARCHAR(255) NOT NULL,
    sql_query_desc      TEXT,
    sql_query_content   TEXT         NOT NULL,
    data_source_id      BIGINT,
    result_meta_updated_at TIMESTAMP NULL,
    is_collection       TINYINT(1)   NOT NULL DEFAULT 1 COMMENT '是否返回多条记录',
    viz_config          JSON         NULL COMMENT '可视化配置 JSON',
    module              VARCHAR(100) NULL COMMENT '功能模块',
    category            VARCHAR(100) NULL COMMENT '分类（可选）',
    version             BIGINT       NOT NULL DEFAULT 1,
    created_by          BIGINT       NOT NULL,
    created_date        TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    BIGINT       NOT NULL,
    last_modified_date  TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by          BIGINT,
    deleted_date        TIMESTAMP NULL,
    CONSTRAINT fk_sql_query_data_source FOREIGN KEY (data_source_id)
        REFERENCES biz_data_source(data_source_id),
    INDEX idx_sql_query_source (data_source_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- 数据源表元数据
CREATE TABLE IF NOT EXISTS biz_data_source_table_meta (
    table_meta_id       BIGINT AUTO_INCREMENT PRIMARY KEY,
    data_source_id      BIGINT        NOT NULL,
    table_name          VARCHAR(255)  NOT NULL,
    comment_label       VARCHAR(500),
    table_desc          TEXT,
    version             BIGINT        NOT NULL DEFAULT 1,
    created_by          BIGINT        NOT NULL,
    created_date        TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    BIGINT        NOT NULL,
    last_modified_date  TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by          BIGINT,
    deleted_date        TIMESTAMP NULL,
    CONSTRAINT fk_table_meta_ds FOREIGN KEY (data_source_id)
        REFERENCES biz_data_source(data_source_id),
    INDEX idx_table_meta_ds (data_source_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- 数据源列元数据
CREATE TABLE IF NOT EXISTS biz_data_source_column_meta (
    column_meta_id      BIGINT AUTO_INCREMENT PRIMARY KEY,
    table_meta_id       BIGINT        NOT NULL,
    column_name         VARCHAR(255)  NOT NULL,
    comment_label       VARCHAR(500),
    field_type          VARCHAR(32)   NOT NULL DEFAULT 'TEXT',
    format_pattern      VARCHAR(100),
    field_desc          TEXT,
    source_physical_type VARCHAR(200),
    ordinal_position    INT           NOT NULL DEFAULT 0 COMMENT '字段原始顺序',
    is_primary_key      INT           NOT NULL DEFAULT 0,
    version             BIGINT        NOT NULL DEFAULT 1,
    created_by          BIGINT        NOT NULL,
    created_date        TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    BIGINT        NOT NULL,
    last_modified_date  TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by          BIGINT,
    deleted_date        TIMESTAMP NULL,
    CONSTRAINT fk_column_meta_table FOREIGN KEY (table_meta_id)
        REFERENCES biz_data_source_table_meta(table_meta_id),
    INDEX idx_column_meta_table (table_meta_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- SQL 查询结果字段
CREATE TABLE IF NOT EXISTS biz_sql_query_result_field (
    result_field_id     BIGINT AUTO_INCREMENT PRIMARY KEY,
    sql_query_id        BIGINT        NOT NULL,
    field_name          VARCHAR(255)  NOT NULL,
    comment_label       VARCHAR(500),
    field_type          VARCHAR(32)   NOT NULL DEFAULT 'TEXT',
    format_pattern      VARCHAR(100),
    field_desc          TEXT,
    version             BIGINT        NOT NULL DEFAULT 1,
    created_by          BIGINT        NOT NULL,
    created_date        TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    BIGINT        NOT NULL,
    last_modified_date  TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by          BIGINT,
    deleted_date        TIMESTAMP NULL,
    CONSTRAINT fk_result_field_query FOREIGN KEY (sql_query_id)
        REFERENCES biz_sql_query(sql_query_id),
    INDEX idx_result_field_query (sql_query_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- SQL 查询参数字段
CREATE TABLE IF NOT EXISTS biz_sql_query_param_field (
    param_field_id      BIGINT AUTO_INCREMENT PRIMARY KEY,
    sql_query_id        BIGINT        NOT NULL,
    field_name          VARCHAR(255)  NOT NULL,
    comment_label       VARCHAR(500),
    field_type          VARCHAR(32)   NOT NULL DEFAULT 'TEXT',
    format_pattern      VARCHAR(100),
    field_desc          TEXT,
    required_flag       TINYINT(1)    NOT NULL DEFAULT 0,
    version             BIGINT        NOT NULL DEFAULT 1,
    created_by          BIGINT        NOT NULL,
    created_date        TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    BIGINT        NOT NULL,
    last_modified_date  TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by          BIGINT,
    deleted_date        TIMESTAMP NULL,
    CONSTRAINT fk_param_field_query FOREIGN KEY (sql_query_id)
        REFERENCES biz_sql_query(sql_query_id),
    INDEX idx_param_field_query (sql_query_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- 看板配置（按用户）
CREATE TABLE IF NOT EXISTS sys_user_dashboard_config (
    user_id            BIGINT       NOT NULL,
    config_json        JSON         NULL,
    version            BIGINT       NOT NULL DEFAULT 1,
    created_by         BIGINT       NOT NULL,
    created_date       TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by   BIGINT       NOT NULL,
    last_modified_date TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by         BIGINT,
    deleted_date       TIMESTAMP NULL,
    PRIMARY KEY (user_id),
    CONSTRAINT fk_dashboard_config_user FOREIGN KEY (user_id)
        REFERENCES sys_user(user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- HTTP API 表
CREATE TABLE IF NOT EXISTS biz_http_api (
    http_api_id         BIGINT AUTO_INCREMENT PRIMARY KEY,
    http_api_name       VARCHAR(255) NOT NULL,
    http_api_path       VARCHAR(255) NOT NULL,
    http_api_desc       TEXT,
    response_template   TEXT,
    module              VARCHAR(100) NULL COMMENT '功能模块',
    category            VARCHAR(100) NULL COMMENT '分类（可选）',
    enabled             TINYINT(1)   NOT NULL DEFAULT 1,
    default_params      JSON         NULL COMMENT '固定参数 JSON（执行时覆盖同名请求参数）',
    version             BIGINT       NOT NULL DEFAULT 1,
    created_by          BIGINT       NOT NULL,
    created_date        TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    BIGINT       NOT NULL,
    last_modified_date  TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_by          BIGINT,
    deleted_date        TIMESTAMP    NULL,
    UNIQUE KEY uk_http_api_path (http_api_path, deleted_date)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- HTTP API 与查询关联表
CREATE TABLE IF NOT EXISTS biz_http_api_query (
    http_api_query_id   BIGINT AUTO_INCREMENT PRIMARY KEY,
    http_api_id         BIGINT       NOT NULL,
    sql_query_id        BIGINT       NOT NULL,
    query_alias         VARCHAR(255) NOT NULL,
    sort_order          BIGINT       NOT NULL DEFAULT 0,
    version             BIGINT       NOT NULL DEFAULT 1,
    created_by          BIGINT       NOT NULL,
    created_date        TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    BIGINT       NOT NULL,
    last_modified_date  TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_by          BIGINT,
    deleted_date        TIMESTAMP    NULL,
    CONSTRAINT fk_http_api_query_api FOREIGN KEY (http_api_id)
        REFERENCES biz_http_api(http_api_id),
    CONSTRAINT fk_http_api_query_sql FOREIGN KEY (sql_query_id)
        REFERENCES biz_sql_query(sql_query_id),
    INDEX idx_http_api_query_api (http_api_id),
    INDEX idx_http_api_query_sql (sql_query_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- AI Provider 配置表
CREATE TABLE IF NOT EXISTS biz_ai_provider (
    ai_provider_id      BIGINT AUTO_INCREMENT PRIMARY KEY,
    name                VARCHAR(64)  NOT NULL,
    base_url            VARCHAR(255) NOT NULL,
    api_key             VARCHAR(512) NOT NULL COMMENT 'AES-GCM 加密后的 base64',
    model               VARCHAR(128) NOT NULL,
    system_prompt       TEXT         NOT NULL,
    temperature         DECIMAL(3,2) NOT NULL DEFAULT 0.30,
    is_default          TINYINT(1)   NOT NULL DEFAULT 0,
    version             BIGINT       NOT NULL DEFAULT 1,
    created_by          BIGINT       NOT NULL,
    created_date        TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    BIGINT       NOT NULL,
    last_modified_date  TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by          BIGINT,
    deleted_date        TIMESTAMP NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- AI 会话表
CREATE TABLE IF NOT EXISTS biz_ai_conversation (
    ai_conversation_id  BIGINT AUTO_INCREMENT PRIMARY KEY,
    sql_query_id        BIGINT       NOT NULL,
    ai_provider_id      BIGINT       NOT NULL,
    title               VARCHAR(128) NOT NULL DEFAULT '新会话',
    version             BIGINT       NOT NULL DEFAULT 1,
    created_by          BIGINT       NOT NULL,
    created_date        TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    BIGINT       NOT NULL,
    last_modified_date  TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by          BIGINT,
    deleted_date        TIMESTAMP NULL,
    INDEX idx_ai_conv_sql_query (sql_query_id),
    INDEX idx_ai_conv_provider (ai_provider_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- AI 消息表
CREATE TABLE IF NOT EXISTS biz_ai_message (
    ai_message_id       BIGINT AUTO_INCREMENT PRIMARY KEY,
    ai_conversation_id  BIGINT       NOT NULL,
    role                VARCHAR(16)  NOT NULL COMMENT 'user / assistant',
    content             MEDIUMTEXT   NOT NULL,
    attached_schema     TINYINT(1)   NOT NULL DEFAULT 0,
    prompt_tokens       INT,
    completion_tokens   INT,
    version             BIGINT       NOT NULL DEFAULT 1,
    created_by          BIGINT       NOT NULL,
    created_date        TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified_by    BIGINT       NOT NULL,
    last_modified_date  TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_by          BIGINT,
    deleted_date        TIMESTAMP NULL,
    INDEX idx_ai_msg_conv (ai_conversation_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
