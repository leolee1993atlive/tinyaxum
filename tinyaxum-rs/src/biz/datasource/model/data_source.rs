use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    common::model::entity::{BaseEntity, BaseVo},
    error::error::AppResult,
    middleware::extractors::CurrentUser,
    util::crypto_util::{decrypt_password, encrypt_password},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataSource {
    pub data_source_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub remark: Option<String>,
    pub db_type: String,
    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
    /// PostgreSQL schema（仅 PG 数据源使用，缺省 public；MySQL 恒为 None）
    pub db_schema: Option<String>,
    pub db_username: String,
    pub db_password: String,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}

impl DataSource {
    pub fn from_create_bo(
        bo: DataSourceCreateBo,
        user: &CurrentUser,
        key: &[u8],
    ) -> AppResult<Self> {
        let encrypted_password = encrypt_password(key, &bo.db_password)?;

        Ok(Self {
            data_source_id: None,
            code: bo.code,
            name: bo.name,
            remark: bo.remark,
            db_type: bo.db_type,
            db_host: bo.db_host,
            db_port: bo.db_port,
            db_name: bo.db_name,
            db_schema: bo.db_schema,
            db_username: bo.db_username,
            db_password: encrypted_password,
            base_entity: BaseEntity::new(user.get_user_id()),
        })
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_update_bo(
        &mut self,
        bo: DataSourceUpdateBo,
        user: &CurrentUser,
        key: &[u8],
    ) -> AppResult<()> {
        // 密码缺省或空串 = 用户想保留当前密码，库里密文原样不动
        if let Some(password) = bo.db_password.filter(|p| !p.is_empty()) {
            self.db_password = encrypt_password(key, &password)?;
        }

        self.data_source_id = bo.data_source_id;
        self.code = bo.code;
        self.name = bo.name;
        self.remark = bo.remark;
        self.db_type = bo.db_type;
        self.db_host = bo.db_host;
        self.db_port = bo.db_port;
        self.db_name = bo.db_name;
        self.db_schema = bo.db_schema;
        self.db_username = bo.db_username;
        self.base_entity.update(user.get_user_id());
        Ok(())
    }

    pub fn to_list_vo(&self) -> DataSourceListVo {
        DataSourceListVo {
            data_source_id: self.data_source_id.unwrap_or_default(),
            code: self.code.clone(),
            name: self.name.clone(),
            remark: self.remark.clone(),
            db_type: self.db_type.clone(),
            db_name: self.db_name.clone(),
            base_entity: BaseVo::new(self.base_entity.clone()),
        }
    }

    /// 对外详情 VO。不含密码，也绝不解密——密钥不匹配时详情照常返回。
    pub fn to_detail_vo(&self) -> DataSourceDetailVo {
        DataSourceDetailVo {
            data_source_id: self.data_source_id.unwrap_or_default(),
            code: self.code.clone(),
            name: self.name.clone(),
            remark: self.remark.clone(),
            db_type: self.db_type.clone(),
            db_host: self.db_host.clone(),
            db_port: self.db_port,
            db_name: self.db_name.clone(),
            db_schema: self.db_schema.clone(),
            db_username: self.db_username.clone(),
            base_entity: BaseVo::new(self.base_entity.clone()),
        }
    }

    /// 服务端内部连接用：解密后的明文密码只在域对象里流转，
    /// 不能进任何 HTTP 响应。
    pub fn to_decrypted(&self, key: &[u8]) -> AppResult<Self> {
        let mut decrypted = self.clone();
        decrypted.db_password = decrypt_password(key, &self.db_password)?;
        Ok(decrypted)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceCreateBo {
    #[validate(length(min = 1, message = "code cannot be empty"))]
    pub code: String,
    #[validate(length(min = 1, message = "name cannot be empty"))]
    pub name: String,
    pub remark: Option<String>,
    #[validate(length(min = 1, message = "db_type cannot be empty"))]
    pub db_type: String,
    #[validate(length(min = 1, message = "db_url cannot be empty"))]
    pub db_host: String,
    pub db_port: u16,
    #[validate(length(min = 1, message = "db_name cannot be empty"))]
    pub db_name: String,
    /// PostgreSQL schema（仅 PG 数据源使用，缺省 public）
    #[validate(length(max = 128, message = "db_schema too long"))]
    pub db_schema: Option<String>,
    #[validate(length(min = 1, message = "db_username cannot be empty"))]
    pub db_username: String,
    #[validate(length(min = 1, message = "db_password cannot be empty"))]
    pub db_password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceUpdateBo {
    #[validate(required)]
    pub data_source_id: Option<i64>,
    #[validate(length(min = 1, message = "code cannot be empty"))]
    pub code: String,
    #[validate(length(min = 1, message = "name cannot be empty"))]
    pub name: String,
    pub remark: Option<String>,
    #[validate(length(min = 1, message = "db_type cannot be empty"))]
    pub db_type: String,
    #[validate(length(min = 1, message = "db_url cannot be empty"))]
    pub db_host: String,
    pub db_port: u16,
    #[validate(length(min = 1, message = "db_name cannot be empty"))]
    pub db_name: String,
    /// PostgreSQL schema（仅 PG 数据源使用，缺省 public）
    #[validate(length(max = 128, message = "db_schema too long"))]
    pub db_schema: Option<String>,
    #[validate(length(min = 1, message = "db_username cannot be empty"))]
    pub db_username: String,
    /// 缺省或空串表示保持当前密码不变（更新专属语义，创建走 CreateBo 必填）。
    pub db_password: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceListVo {
    pub data_source_id: i64,
    pub code: String,
    pub name: String,
    pub remark: Option<String>,
    pub db_type: String,
    pub db_name: String,
    #[serde(flatten)]
    pub base_entity: BaseVo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceDetailVo {
    pub data_source_id: i64,
    pub code: String,
    pub name: String,
    pub remark: Option<String>,
    pub db_type: String,
    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
    pub db_schema: Option<String>,
    pub db_username: String,
    #[serde(flatten)]
    pub base_entity: BaseVo,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const KEY: &[u8] = b"01234567890123456789012345678901";

    fn test_user() -> CurrentUser {
        CurrentUser {
            id: 1,
            username: "tester".to_string(),
        }
    }

    fn sample() -> DataSource {
        DataSource::from_create_bo(
            DataSourceCreateBo {
                code: "ds".to_string(),
                name: "数据源".to_string(),
                remark: None,
                db_type: "mysql".to_string(),
                db_host: "10.0.0.8".to_string(),
                db_port: 3307,
                db_name: "demo".to_string(),
                db_schema: None,
                db_username: "root".to_string(),
                db_password: "s3cret".to_string(),
            },
            &test_user(),
            KEY,
        )
        .expect("构造数据源失败")
    }

    fn update_bo(db_password: Option<String>) -> DataSourceUpdateBo {
        DataSourceUpdateBo {
            data_source_id: Some(1),
            code: "ds".to_string(),
            name: "数据源".to_string(),
            remark: None,
            db_type: "mysql".to_string(),
            db_host: "10.0.0.8".to_string(),
            db_port: 3307,
            db_name: "demo".to_string(),
            db_schema: None,
            db_username: "root".to_string(),
            db_password,
        }
    }

    #[test]
    fn detail_vo_never_carries_password() {
        let vo = sample().to_detail_vo();
        let serialized = serde_json::to_value(&vo).unwrap();
        assert!(
            serialized.get("password").is_none()
                && serialized.get("dbPassword").is_none()
                && !serialized.to_string().contains("s3cret"),
            "详情响应不得出现任何密码字段: {serialized}"
        );
    }

    #[test]
    fn to_decrypted_roundtrips_plaintext() {
        let ds = sample();
        assert_eq!(ds.to_decrypted(KEY).unwrap().db_password, "s3cret");

        let wrong_key = b"11234567890123456789012345678901";
        assert!(ds.to_decrypted(wrong_key).is_err());
    }

    #[test]
    fn update_with_blank_password_keeps_current_ciphertext() {
        let user = test_user();
        let original = sample();
        let original_cipher = original.db_password.clone();

        for password in [None, Some(String::new())] {
            let mut updated = original.clone();
            updated
                .from_update_bo(update_bo(password), &user, KEY)
                .unwrap();
            assert_eq!(
                updated.db_password, original_cipher,
                "密码留空必须保持库中密文不变"
            );
        }
    }

    #[test]
    fn update_with_blank_password_still_updates_other_fields() {
        let mut ds = sample();
        let original_cipher = ds.db_password.clone();

        let mut bo = update_bo(Some(String::new()));
        bo.db_host = "172.16.0.1".to_string();
        ds.from_update_bo(bo, &test_user(), KEY).unwrap();

        assert_eq!(ds.db_host, "172.16.0.1");
        assert_eq!(ds.db_password, original_cipher);
    }

    #[test]
    fn update_with_new_password_reencrypts() {
        let mut ds = sample();
        ds.from_update_bo(
            update_bo(Some("new-pass".to_string())),
            &test_user(),
            KEY,
        )
        .unwrap();

        assert_ne!(ds.db_password, "new-pass");
        assert_eq!(ds.to_decrypted(KEY).unwrap().db_password, "new-pass");
    }

    #[test]
    fn create_bo_serializes_camel_case_password() {
        let bo = DataSourceCreateBo {
            code: "ds".to_string(),
            name: "数据源".to_string(),
            remark: None,
            db_type: "mysql".to_string(),
            db_host: "10.0.0.8".to_string(),
            db_port: 3307,
            db_name: "demo".to_string(),
            db_schema: None,
            db_username: "root".to_string(),
            db_password: String::new(),
        };
        let serialized = serde_json::to_value(&bo).unwrap();
        assert_eq!(
            serialized,
            json!({
                "code": "ds", "name": "数据源", "remark": null,
                "dbType": "mysql", "dbHost": "10.0.0.8", "dbPort": 3307,
                "dbName": "demo", "dbSchema": null, "dbUsername": "root", "dbPassword": ""
            })
        );
    }
}
