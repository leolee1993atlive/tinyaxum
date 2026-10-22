use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode,
    errors::ErrorKind,
};
use serde::{Deserialize, Serialize};

use crate::error::error::AppError::{self, JwtTokenError};

const ACCESS_TOKEN_TYPE: &str = "access";
const REFRESH_TOKEN_TYPE: &str = "refresh";
const JWT_AUDIENCE: &str = "rust_admin";
const JWT_ISSUER: &str = "koobe";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtToken {
    pub id: i64,
    pub username: String,
    pub permissions: Vec<String>,
    pub token_type: String,
    aud: String,
    // (audience)：受众
    exp: usize,
    iat: usize,
    // (Issued At)：签发时间
    iss: String,
    // (issuer)：签发人
    nbf: usize,
    // (Not Before)：生效时间
    sub: String,
    // (subject)：主题
    jti: String, // (JWT ID)：编号
}

impl JwtToken {
    fn current_timestamp() -> Result<Duration, AppError> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| JwtTokenError(format!("获取系统时间失败: {}", err)))
    }

    fn build(
        user_id: i64,
        username: &str,
        permissions: Vec<String>,
        token_type: &str,
        expires_in: Duration,
    ) -> Result<JwtToken, AppError> {
        let now = Self::current_timestamp()?;

        Ok(JwtToken {
            id: user_id,
            username: String::from(username),
            permissions,
            token_type: token_type.to_string(),
            aud: String::from(JWT_AUDIENCE), // (audience)：受众
            exp: (now + expires_in).as_secs() as usize,
            iat: now.as_secs() as usize, // (Issued At)：签发时间
            iss: String::from(JWT_ISSUER), // (issuer)：签发人
            nbf: now.as_secs() as usize, // (Not Before)：生效时间
            sub: user_id.to_string(),    // (subject)：主题
            jti: "ignore".to_string(),   // (JWT ID)：编号
        })
    }

    pub fn new_access(
        user_id: i64,
        username: &str,
        permissions: Vec<String>,
        ttl_secs: u64,
    ) -> Result<JwtToken, AppError> {
        Self::build(
            user_id,
            username,
            permissions,
            ACCESS_TOKEN_TYPE,
            Duration::from_secs(ttl_secs),
        )
    }

    pub fn new_refresh(
        user_id: i64,
        username: &str,
        ttl_secs: u64,
    ) -> Result<JwtToken, AppError> {
        Self::build(
            user_id,
            username,
            Vec::new(),
            REFRESH_TOKEN_TYPE,
            Duration::from_secs(ttl_secs),
        )
    }

    /// create token
    /// secret: your secret string
    pub fn create_token(&self, secret: &str) -> Result<String, AppError> {
        match encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(secret.as_ref()),
        ) {
            Ok(t) => Ok(t),
            Err(e) => {
                tracing::error!(
                    error = %e,
                    user_id = self.id,
                    username = %self.username,
                    "JWT 令牌编码失败"
                );
                Err(JwtTokenError("创建令牌失败".to_string()))
            }
        }
    }

    /// verify token invalid
    /// secret: your secret string
    pub fn verify(secret: &str, token: &str) -> Result<JwtToken, AppError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&[JWT_AUDIENCE]);
        validation.set_required_spec_claims(&["exp", "sub", "aud"]);
        match decode::<JwtToken>(
            &token,
            &DecodingKey::from_secret(secret.as_ref()),
            &validation,
        ) {
            Ok(c) => Ok(c.claims),

            Err(err) => match err.kind() {
                ErrorKind::InvalidToken => {
                    tracing::error!(
                        error_kind = "InvalidToken",
                        "JWT 令牌验证失败: 令牌无效"
                    );
                    Err(JwtTokenError("令牌无效".to_string()))
                }
                ErrorKind::InvalidIssuer => {
                    tracing::error!(
                        error_kind = "InvalidIssuer",
                        "JWT 令牌验证失败: 签发者无效"
                    );
                    Err(JwtTokenError("签发者无效".to_string()))
                }
                ErrorKind::ExpiredSignature => {
                    tracing::error!(
                        error_kind = "ExpiredSignature",
                        "JWT 令牌验证失败: 令牌已过期"
                    );
                    Err(JwtTokenError("token 已经超时了".to_string()))
                }
                other => {
                    tracing::error!(
                        error_kind = %format!("{:?}", other),
                        "JWT 令牌验证失败"
                    );
                    Err(JwtTokenError("创建令牌失败".to_string()))
                }
            },
        }
    }

    pub fn is_access_token(&self) -> bool {
        self.token_type == ACCESS_TOKEN_TYPE
    }

    pub fn is_refresh_token(&self) -> bool {
        self.token_type == REFRESH_TOKEN_TYPE
    }
}
