use std::sync::Arc;

use axum::{
    extract::{FromRef, FromRequestParts},
    http,
    http::request::Parts,
};

use crate::{app::AppState, error::error::AppError, util::jwt_util::JwtToken};

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: i64,
    pub username: String,
}

impl CurrentUser {
    pub fn get_user_id(&self) -> i64 {
        self.id
    }
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        if let Some(jwt_token) = parts.extensions.get::<JwtToken>() {
            return Ok(CurrentUser {
                id: jwt_token.id,
                username: jwt_token.username.clone(),
            });
        }

        let auth_header = parts
            .headers
            .get(http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        let token = match auth_header {
            Some(header) => {
                header.strip_prefix("Bearer ").ok_or_else(|| {
                    tracing::warn!("CurrentUser 提取失败: token 格式错误");
                    AppError::JwtTokenError("token格式错误".to_string())
                })?
            }
            None => {
                let cookie_str = parts
                    .headers
                    .get(http::header::COOKIE)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");
                cookie_str
                    .split(';')
                    .find_map(|c| {
                        let c = c.trim();
                        c.strip_prefix("access_token=")
                    })
                    .ok_or_else(|| {
                        tracing::warn!(
                            "CurrentUser 提取失败: 无 Authorization 头或 access_token cookie"
                        );
                        AppError::JwtTokenError(String::from(
                            "请求头缺少 Authorization 字段",
                        ))
                    })?
            }
        };

        let app_state = Arc::<AppState>::from_ref(_state);

        let claims = JwtToken::verify(&app_state.infra.jwt_secret, token)
            .map_err(|e| {
                tracing::warn!(error = %e, "CurrentUser 提取失败: JWT 验证失败");
                e
            })?;

        Ok(CurrentUser {
            id: claims.id,
            username: claims.username,
        })
    }
}
