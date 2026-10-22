-- =============================================================================
-- 0004 · 整型列 DDL 与实体类型对齐
-- 适用：已按旧版 0001_init 建库的存量环境（新库直接用修正后的 0001_init，无需本脚本）。
-- 背景：本栈 sea-orm 2.0.0-rc + sqlx-postgres 0.8.6 整型解码按 OID 精确匹配、
--       零转换（i64←INT8 / i32←INT4 / i16←INT2 / i8←"char"），DDL 类型与实体
--       不符会直接报 mismatched types；MySQL 侧另因 SMALLINT 有符号上限 32767
--       存不下 33106 这类端口值（导入 biz_data_source 报 smallint 超范围）。
-- 对齐基准（实体字段类型）：
--       i32：biz_data_source.db_port、biz_data_source_column_meta.is_primary_key、
--            biz_sql_query_param_field.required_flag、biz_http_api.enabled
--       i8 ：biz_sql_query.is_collection、biz_ai_provider.is_default、
--            biz_ai_message.attached_schema（PG 存 "char"，注意 0/1 必须是字节值
--            0/1 而非 ASCII '0'/'1'=48/49，故用 ::int::"char" 转换）
-- =============================================================================

-- PostgreSQL（执行）
-- i32 列：直接改类型（重复执行为无害空操作）
ALTER TABLE biz_data_source              ALTER COLUMN db_port        TYPE INTEGER;
ALTER TABLE biz_data_source_column_meta  ALTER COLUMN is_primary_key TYPE INTEGER;
ALTER TABLE biz_sql_query_param_field    ALTER COLUMN required_flag  TYPE INTEGER;
ALTER TABLE biz_http_api                 ALTER COLUMN enabled        TYPE INTEGER;
-- i8 列：USING 只转换列数据，不转换旧默认值（整型 0/1 无法自动转 "char"，
-- 会报"默认值不能转换成类型 char"），必须先 DROP DEFAULT 再改类型再重设
ALTER TABLE biz_sql_query   ALTER COLUMN is_collection   DROP DEFAULT;
ALTER TABLE biz_ai_provider ALTER COLUMN is_default      DROP DEFAULT;
ALTER TABLE biz_ai_message  ALTER COLUMN attached_schema DROP DEFAULT;
ALTER TABLE biz_sql_query   ALTER COLUMN is_collection   TYPE "char" USING is_collection::int::"char";
ALTER TABLE biz_ai_provider ALTER COLUMN is_default      TYPE "char" USING is_default::int::"char";
ALTER TABLE biz_ai_message  ALTER COLUMN attached_schema TYPE "char" USING attached_schema::int::"char";
ALTER TABLE biz_sql_query   ALTER COLUMN is_collection   SET DEFAULT 1::int::"char";
ALTER TABLE biz_ai_provider ALTER COLUMN is_default      SET DEFAULT 0::int::"char";
ALTER TABLE biz_ai_message  ALTER COLUMN attached_schema SET DEFAULT 0::int::"char";

-- MySQL（仅 db_port 有数据范围问题，其余 TINYINT(1)/INT 与实体解码兼容；执行）
ALTER TABLE biz_data_source MODIFY COLUMN db_port INT NOT NULL;

-- SQLite（无需执行：INTEGER 亲和 + sqlx sqlite 整型解码宽松，与实体不受影响）
