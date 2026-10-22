-- =============================================================
-- TinyAxum 初始化脚本 (PostgreSQL 版本)
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
