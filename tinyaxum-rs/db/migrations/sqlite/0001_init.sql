-- =============================================================
-- TinyAxum 初始化脚本 (SQLite 兼容版)
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
