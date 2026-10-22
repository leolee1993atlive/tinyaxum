use std::sync::Arc;

use axum::{
    extract::{MatchedPath, Request, State},
    http,
    http::StatusCode,
    middleware::Next,
    response,
    response::IntoResponse,
};

use crate::{
    app::AppState,
    common::vo::response::R,
    error::error::AppError,
    util::{jwt_util::JwtToken, metrics},
};

#[tracing::instrument(skip(state, req, next))]
pub async fn auth(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<response::Response, StatusCode> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    tracing::debug!(method = %method, uri = %uri, "收到请求");

    let metrics_path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|mp| mp.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string());

    let raw_path = req.uri().path().to_string();
    if raw_path.eq("/login")
        || raw_path.eq("/refreshToken")
        || raw_path.eq("/logout")
        || raw_path.starts_with("/public/")
    {
        let _timer = metrics::RequestTimer::new(
            method.to_string(),
            metrics_path.clone(),
        );
        let resp = next.run(req).await;
        metrics::record_http_request(
            method.as_str(),
            &metrics_path,
            resp.status().as_u16(),
        );
        return Ok(resp);
    }

    let token = match extract_token(&req) {
        Some(t) => t,
        None => {
            tracing::warn!(uri = %uri, "请求缺少认证凭据");
            metrics::record_http_request(method.as_str(), &metrics_path, 401);
            return Ok(R::<()>::error_with_code_and_message(
                401,
                "请求缺少认证凭据".to_string(),
            )
            .into_response());
        }
    };

    let jwt_token = match JwtToken::verify(&state.infra.jwt_secret, &token) {
        Ok(data) => data,
        Err(err) => {
            let er = match err {
                AppError::JwtTokenError(s) => s,
                _ => "未知错误".to_string(),
            };
            tracing::warn!(uri = %uri, error = %er, "Token 校验失败");
            metrics::record_http_request(method.as_str(), &metrics_path, 401);
            return Ok(R::<()>::error_with_code_and_message(
                401,
                format!("Token 校验失败: {}", er),
            )
            .into_response());
        }
    };

    if !jwt_token.is_access_token() {
        tracing::warn!(uri = %uri, "access token 类型无效");
        metrics::record_http_request(method.as_str(), &metrics_path, 401);
        return Ok(R::<()>::error_with_code_and_message(
            401,
            "access token 类型无效".to_string(),
        )
        .into_response());
    }

    let has_permission = jwt_token.permissions.iter().any(|p| {
        p == "*"
            || p == raw_path.as_str()
            || p.strip_prefix("/api").unwrap_or(p) == raw_path
    });

    if has_permission {
        if let Ok(val) = jwt_token.id.to_string().parse() {
            req.headers_mut().insert("user_id", val);
        } else {
            tracing::error!(uri = %uri, user_id = %jwt_token.id, "user_id 解析失败");
        }
        req.extensions_mut().insert(jwt_token);
        let _timer = metrics::RequestTimer::new(
            method.to_string(),
            metrics_path.clone(),
        );
        let resp = next.run(req).await;
        metrics::record_http_request(
            method.as_str(),
            &metrics_path,
            resp.status().as_u16(),
        );
        return Ok(resp);
    }

    tracing::warn!(uri = %uri, "用户未授权访问");
    metrics::record_http_request(method.as_str(), &metrics_path, 401);
    Ok(R::<()>::error_with_code_and_message(
        401,
        format!("用户还没有授权url:{raw_path}"),
    )
    .into_response())
}

fn extract_token(req: &Request) -> Option<String> {
    if let Some(t) = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
    {
        return Some(t.to_string());
    }

    let cookie_header = req
        .headers()
        .get(http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    cookie_header.split(';').find_map(|cookie| {
        let cookie = cookie.trim();
        cookie.strip_prefix("access_token=").map(|v| v.to_string())
    })
}
