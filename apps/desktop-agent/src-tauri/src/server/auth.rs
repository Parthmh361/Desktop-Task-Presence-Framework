use std::net::SocketAddr;

use axum::http::HeaderMap;

use crate::AppState;

pub struct AuthContext {
    pub app_id: String,
    pub origin: String,
}

pub async fn authenticate_request(
    state: &AppState,
    addr: &SocketAddr,
    headers: &HeaderMap,
) -> Result<AuthContext, String> {
    if !is_localhost(addr) {
        return Err("Requests must originate from localhost".into());
    }

    let app_id = headers
        .get("x-dtpf-app-id")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| "Missing X-DTPF-App-ID header".to_string())?;

    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| "Missing Authorization header".to_string())?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| "Invalid Authorization header format".to_string())?;

    authenticate_with_token(state, addr, app_id, token, headers.get("origin")).await
}

pub async fn authenticate_with_token(
    state: &AppState,
    addr: &SocketAddr,
    app_id: &str,
    token: &str,
    origin_header: Option<&axum::http::HeaderValue>,
) -> Result<AuthContext, String> {
    if !is_localhost(addr) {
        return Err("Requests must originate from localhost".into());
    }

    let registration = state
        .db
        .get_app_by_token(token)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Invalid token".to_string())?;

    if registration.app_id != app_id {
        return Err("Token does not match app ID".into());
    }

    let origin = origin_header
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !origin.is_empty() && origin != registration.origin {
        return Err("Origin mismatch".into());
    }

    state.db.touch_app(app_id).await.ok();

    Ok(AuthContext {
        app_id: app_id.to_string(),
        origin: registration.origin,
    })
}

fn is_localhost(addr: &SocketAddr) -> bool {
    match addr.ip() {
        std::net::IpAddr::V4(v4) => v4.is_loopback(),
        std::net::IpAddr::V6(v6) => v6.is_loopback(),
    }
}
