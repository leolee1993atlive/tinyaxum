//! 业务语义字段类型（与库中 `field_type` VARCHAR 一致）

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FieldSemanticType {
    Text,
    Numeric,
    Temporal,
    FieldFilter,
}

impl FieldSemanticType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "TEXT",
            Self::Numeric => "NUMERIC",
            Self::Temporal => "TEMPORAL",
            Self::FieldFilter => "FIELD_FILTER",
        }
    }

    /// 根据 MySQL `information_schema.COLUMNS.DATA_TYPE` 粗映射
    pub fn from_mysql_data_type(data_type: &str) -> Self {
        match data_type.to_lowercase().as_str() {
            "tinyint" | "smallint" | "mediumint" | "int" | "integer"
            | "bigint" | "decimal" | "float" | "double" | "real" => {
                Self::Numeric
            }
            "date" | "time" | "datetime" | "timestamp" | "year" => {
                Self::Temporal
            }
            _ => Self::Text,
        }
    }

    /// 根据 PostgreSQL `information_schema.columns.data_type` 粗映射
    pub fn from_pg_data_type(data_type: &str) -> Self {
        match data_type.to_lowercase().as_str() {
            "smallint" | "integer" | "bigint" | "decimal" | "numeric"
            | "real" | "double precision" | "serial" | "bigserial" => {
                Self::Numeric
            }
            "date"
            | "time"
            | "time without time zone"
            | "time with time zone"
            | "timestamp"
            | "timestamp without time zone"
            | "timestamp with time zone" => Self::Temporal,
            _ => Self::Text,
        }
    }
}
