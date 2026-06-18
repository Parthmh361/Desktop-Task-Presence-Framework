use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Query, State, WebSocketUpgrade},
    http::HeaderMap,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::server::auth::authenticate_with_token;
use crate::server::routes::ApiError;
use crate::AppState;

#[derive(Deserialize)]
pub struct WsQuery {
    token: Option<String>,
    app_id: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<std::sync::Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(query): Query<WsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(String::from)
        .or(query.token);

    let app_id = headers
        .get("x-dtpf-app-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .or(query.app_id);

    let (token, app_id) = match (token, app_id) {
        (Some(t), Some(a)) => (t, a),
        _ => return Err(ApiError::Unauthorized("Missing auth for WebSocket".into())),
    };

    let auth = authenticate_with_token(
        &state,
        &addr,
        &app_id,
        &token,
        headers.get("origin"),
    )
    .await
    .map_err(|e| ApiError::Unauthorized(e))?;

    if !state.rate_limiter.check(&auth.app_id) {
        return Err(ApiError::RateLimited);
    }

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, auth.app_id)))
}

async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    state: std::sync::Arc<AppState>,
    app_id: String,
) {
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};

    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.event_tx.subscribe();
    let app_id_filter = app_id.clone();

    let send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    if !event_matches_app(&msg, &app_id_filter) {
                        continue;
                    }
                    if sender.send(Message::Text(msg)).await.is_err() {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => break,
            }
        }
    });

    while let Some(Ok(msg)) = receiver.next().await {
        if matches!(msg, Message::Close(_)) {
            break;
        }
    }

    send_task.abort();
    let _ = state.db.touch_app(&app_id).await;
}

fn event_matches_app(msg: &str, app_id: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(msg) else {
        return false;
    };
    value
        .get("sourceAppId")
        .and_then(|v| v.as_str())
        .map(|id| id == app_id)
        .unwrap_or(false)
}
