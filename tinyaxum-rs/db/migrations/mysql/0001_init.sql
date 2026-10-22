-- =============================================================
-- TinyAxum 初始化脚本 (MySQL 版本)
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
