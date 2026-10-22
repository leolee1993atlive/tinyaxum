use std::sync::Arc;

use axum::{Json, extract::State};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use cookie::time::Duration as CookieDuration;
use validator::Validate;

use crate::{
    app::AppState,
    auth::model::auth::{AuthBody, AuthPayload, TokenPayload},
    common::vo::response::R,
    error::error::AppError,
    util::jwt_util::JwtToken,
};

const ALL_PERMISSIONS: [&str; 1] = ["*"];

fn default_permissions() -> Vec<String> {
    ALL_PERMISSIONS
        .iter()
        .map(|permission| permission.to_string())
        .collect()
}

fn build_access_cookie(token: &str, ttl: u64) -> Cookie<'static> {
    Cookie::build(("access_token", token.to_string()))
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(CookieDuration::seconds(ttl as i64))
        .build()
}

fn build_refresh_cookie(token: &str, ttl: u64) -> Cookie<'static> {
    Cookie::build(("refresh_token", token.to_string()))
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(CookieDuration::seconds(ttl as i64))
        .build()
}

#[tracing::instrument(skip(payload, state))]
pub async fn login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(payload): Json<AuthPayload>,
) -> Result<(CookieJar, R<AuthBody>), AppError> {
    payload.validate()?;

    tracing::info!(username = %payload.username, "尝试登录");
    let user = state
        .services
        .user_service
        .get_user_by_username(&payload.username)
        .await
        .map_err(|e| match e {
            AppError::NotFound(_) => {
                tracing::warn!(
                    username = %payload.username,
                    "用户不存在"
                );
                AppError::Auth("用户或密码错误".to_string())
            }
            other => other,
        })?;

    if !bcrypt::verify(payload.password.as_bytes(), &user.password)
        .unwrap_or(false)
    {
        tracing::warn!(username = %payload.username, "密码错误");
        return Err(AppError::Auth("用户或密码错误".to_string()));
    }

    let jwt_secret = &state.infra.jwt_secret;
    let user_id = match user.user_id {
        Some(id) => id,
        None => {
            tracing::error!(
                username = %payload.username,
                "用户ID为空"
            );
            return Err(AppError::Internal("用户ID为空".to_string()));
        }
    };
    let permissions = default_permissions();

    let token = JwtToken::new_access(
        user_id,
        &user.username,
        permissions,
        state.infra.jwt_access_ttl,
    )
    .map_err(|e| {
        tracing::error!(
            user_id = %user_id,
            error = %e,
            "创建 access token 失败"
        );
        AppError::Internal(format!("创建token失败: {}", e))
    })?;
    let token_str = token.create_token(jwt_secret).map_err(|e| {
        tracing::error!(
            user_id = %user_id,
            error = %e,
            "编码 access token 失败"
        );
        AppError::Internal(format!("创建token失败: {}", e))
    })?;
    let refresh_token = JwtToken::new_refresh(
        user_id,
        &user.username,
        state.infra.jwt_refresh_ttl,
    )
    .map_err(|e| {
        tracing::error!(
            user_id = %user_id,
            error = %e,
            "创建 refresh token 失败"
        );
        AppError::Internal(format!("创建refresh token失败: {}", e))
    })?
    .create_token(jwt_secret)
    .map_err(|e| {
        tracing::error!(
            user_id = %user_id,
            error = %e,
            "编码 refresh token 失败"
        );
        AppError::Internal(format!("创建refresh token失败: {}", e))
    })?;
    tracing::info!(
        user_id = %user_id,
        username = %payload.username,
        "登录成功"
    );

    let updated_jar = jar
        .add(build_access_cookie(&token_str, state.infra.jwt_access_ttl))
        .add(build_refresh_cookie(
            &refresh_token,
            state.infra.jwt_refresh_ttl,
        ));

    Ok((
        updated_jar,
        R::ok_with_data(AuthBody::new(token_str, refresh_token)),
    ))
}

#[tracing::instrument(skip(state, jar, body))]
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    body: Option<Json<TokenPayload>>,
) -> Result<(CookieJar, R<AuthBody>), AppError> {
    let refresh_token_str = jar
        .get("refresh_token")
        .map(|c| c.value().to_string())
        .or_else(|| body.and_then(|Json(p)| p.refresh_token))
        .ok_or_else(|| AppError::Auth("未提供 refresh token".to_string()))?;

    let jwt_secret = &state.infra.jwt_secret;
    let jwt_token = match JwtToken::verify(jwt_secret, &refresh_token_str) {
        Ok(data) => data,
        Err(err) => {
            tracing::error!(error = %err, "refresh token 验证失败");
            return Err(AppError::Auth(format!("Token 校验失败: {}", err)));
        }
    };

    if !jwt_token.is_refresh_token() {
        tracing::warn!(
            username = %jwt_token.username,
            "token 不是 refresh token"
        );
        return Err(AppError::Auth("refresh token 类型无效".to_string()));
    }

    let token = JwtToken::new_access(
        jwt_token.id,
        jwt_token.username.as_str(),
        default_permissions(),
        state.infra.jwt_access_ttl,
    )
    .map_err(|e| {
        tracing::error!(
            user_id = %jwt_token.id,
            error = %e,
            "刷新时创建 access token 失败"
        );
        AppError::Internal(format!("创建token失败: {}", e))
    })?;
    let token_str = token.create_token(jwt_secret).map_err(|e| {
        tracing::error!(
            user_id = %jwt_token.id,
            error = %e,
            "刷新时编码 access token 失败"
        );
        AppError::Internal(format!("创建token失败: {}", e))
    })?;
    let next_refresh_token = JwtToken::new_refresh(
        jwt_token.id,
        jwt_token.username.as_str(),
        state.infra.jwt_refresh_ttl,
    )
    .map_err(|e| {
        tracing::error!(
            user_id = %jwt_token.id,
            error = %e,
            "刷新时创建 refresh token 失败"
        );
        AppError::Internal(format!("创建refresh token失败: {}", e))
    })?
    .create_token(jwt_secret)
    .map_err(|e| {
        tracing::error!(
            user_id = %jwt_token.id,
            error = %e,
            "刷新时编码 refresh token 失败"
        );
        AppError::Internal(format!("创建refresh token失败: {}", e))
    })?;
    tracing::info!(
        user_id = %jwt_token.id,
        username = %jwt_token.username,
        "token 刷新成功"
    );

    let updated_jar = jar
        .add(build_access_cookie(&token_str, state.infra.jwt_access_ttl))
        .add(build_refresh_cookie(
            &next_refresh_token,
            state.infra.jwt_refresh_ttl,
        ));

    Ok((
        updated_jar,
        R::ok_with_data(AuthBody::new(token_str, next_refresh_token)),
    ))
}

#[tracing::instrument(skip(state, jar, headers))]
pub async fn logout(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    headers: axum::http::HeaderMap,
) -> Result<(CookieJar, R<()>), AppError> {
    if let Ok(token) = extract_token_from_request(&headers, &jar) {
        if let Ok(jwt_token) = JwtToken::verify(&state.infra.jwt_secret, &token)
        {
            tracing::info!(username = %jwt_token.username, "用户登出成功");
        } else {
            tracing::warn!("logout 时 token 验证失败，但仍清除 cookie");
        }
    }

    let cleared_jar = jar
        .remove(Cookie::from("access_token"))
        .remove(Cookie::from("refresh_token"));

    Ok((cleared_jar, R::ok()))
}

fn extract_token_from_request(
    headers: &axum::http::HeaderMap,
    jar: &CookieJar,
) -> Result<String, AppError> {
    if let Some(t) = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
    {
        return Ok(t.to_string());
    }
    jar.get("access_token")
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::Auth("未提供认证凭据".to_string()))
}
