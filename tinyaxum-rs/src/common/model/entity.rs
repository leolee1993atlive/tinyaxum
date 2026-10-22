use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEntity {
    pub version: i64,
    pub created_by: i64,
    pub created_date: DateTime<Utc>,
    pub last_modified_by: i64,
    pub last_modified_date: DateTime<Utc>,
    pub deleted_by: Option<i64>,
    pub deleted_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseVo {
    pub version: i64,
    pub created_by: i64,
    pub created_date: DateTime<Utc>,
    pub last_modified_by: i64,
    pub last_modified_date: DateTime<Utc>,
    pub deleted_by: Option<i64>,
    pub deleted_date: Option<DateTime<Utc>>,
}

impl BaseEntity {
    pub fn new(created_by: i64) -> Self {
        Self {
            version: 1,
            created_by,
            created_date: Utc::now(),
            last_modified_by: created_by,
            last_modified_date: Utc::now(),
            deleted_by: None,
            deleted_date: None,
        }
    }

    pub fn update(&mut self, last_modified_by: i64) {
        self.version += 1;
        self.last_modified_by = last_modified_by;
        self.last_modified_date = Utc::now();
    }

    pub fn delete(&mut self, deleted_by: i64) {
        self.deleted_by = Some(deleted_by);
        self.deleted_date = Some(Utc::now());
    }
}

impl BaseVo {
    pub fn new(base_entity: BaseEntity) -> Self {
        Self {
            version: base_entity.version,
            created_by: base_entity.created_by,
            created_date: base_entity.created_date,
            last_modified_by: base_entity.last_modified_by,
            last_modified_date: base_entity.last_modified_date,
            deleted_by: base_entity.deleted_by,
            deleted_date: base_entity.deleted_date,
        }
    }
}
