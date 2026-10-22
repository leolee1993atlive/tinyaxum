-- =============================================================================
-- 0002 · biz_http_api 增加 default_params（固定参数）列
-- 适用：已按 0001_init 建库的存量环境（新库直接用 0001_init，无需本脚本）。
-- 用途：公开 API 可配置固定参数（JSON 对象），执行时覆盖同名请求参数，
--       用于同一参数化查询支撑多个口径不同的旧接口（天数窗口/ICD 列表/写死科室等）。
-- 代码版本：>= 本次发布（HttpApi.default_params / execute_public_api 合并逻辑）
-- =============================================================================

-- MySQL
ALTER TABLE biz_http_api
    ADD COLUMN default_params JSON NULL COMMENT '固定参数 JSON（执行时覆盖同名请求参数）' AFTER enabled;

-- PostgreSQL（二选一，按实际库执行）
-- ALTER TABLE biz_http_api ADD COLUMN default_params JSONB NULL;

-- SQLite
-- ALTER TABLE biz_http_api ADD COLUMN default_params TEXT NULL;
