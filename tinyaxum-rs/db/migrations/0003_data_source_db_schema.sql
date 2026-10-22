-- =============================================================================
-- 0003 · biz_data_source 增加 db_schema 列
-- 适用：已按 0001_init 建库的存量环境（新库直接用 0001_init，无需本脚本）。
-- 用途：PostgreSQL 数据源显式指定 schema（数据同步/元数据采集不再猜测，
--       缺省回退 public）；MySQL 数据源该列保持 NULL，不使用。
-- 代码版本：>= 本次发布（DataSource.db_schema / introspect_postgresql /
--       DbToDuckdbSync 显式 schema 引用）
-- =============================================================================

-- MySQL
ALTER TABLE biz_data_source
    ADD COLUMN db_schema VARCHAR(128) NULL COMMENT 'PG schema（仅 PostgreSQL 用，缺省 public）' AFTER db_name;
UPDATE biz_data_source SET db_schema = 'public'
 WHERE db_type IN ('postgres', 'postgresql') AND db_schema IS NULL;

-- PostgreSQL（二选一，按实际库执行）
-- ALTER TABLE biz_data_source ADD COLUMN db_schema VARCHAR(128) NULL;
-- UPDATE biz_data_source SET db_schema = 'public'
--  WHERE db_type IN ('postgres', 'postgresql') AND db_schema IS NULL;

-- SQLite
-- ALTER TABLE biz_data_source ADD COLUMN db_schema VARCHAR(128) NULL;
-- UPDATE biz_data_source SET db_schema = 'public'
--  WHERE db_type IN ('postgres', 'postgresql') AND db_schema IS NULL;
