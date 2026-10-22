use serde::{Deserialize, Serialize};
use validator::Validate;

impl AuthBody {
    pub fn new(access_token: String, refresh_token: String) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthBody {
    access_token: String,
    refresh_token: String,
    token_type: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AuthPayload {
    #[validate(length(
        min = 1,
        max = 50,
        message = "用户名不能为空且不超过50字符"
    ))]
    pub username: String,
    #[validate(length(
        min = 1,
        max = 100,
        message = "密码不能为空且不超过100字符"
    ))]
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPayload {
    #[serde(default)]
    pub refresh_token: Option<String>,
}
