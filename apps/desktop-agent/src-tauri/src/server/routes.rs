use std::sync::Arc;

use axum::{
    extract::{ConnectInfo, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::db::{NewTask, TaskUpdate};
use crate::server::auth::{authenticate_request, AuthContext};
use crate::server::{register_app_with_approval, task_row_to_json, VERSION};
use crate::window_manager;
use crate::AppState;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub app_id: String,
    pub app_name: String,
    pub origin: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub token: String,
    pub app_id: String,
}

#[derive(Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub body: Option<String>,
    pub priority: Option<i64>,
    pub color: Option<String>,
    pub remind_at: Option<i64>,
    pub position: Option<PositionRequest>,
    pub monitor_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct PositionRequest {
    pub x: i64,
    pub y: i64,
}

#[derive(Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub body: Option<String>,
    pub priority: Option<i64>,
    pub color: Option<String>,
    pub remind_at: Option<i64>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct SnoozeRequest {
    pub until: i64,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(register))
        .route("/tasks", get(list_tasks).post(create_task))
        .route(
            "/tasks/:id",
            get(get_task).put(update_task).delete(delete_task),
        )
        .route("/tasks/:id/complete", post(complete_task))
        .route("/tasks/:id/snooze", post(snooze_task))
        .route("/tasks/:id/remind", post(remind_task))
        .route("/ws", get(crate::server::websocket::ws_handler))
        .with_state(state)
}

async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let task_count = state.db.count_active_tasks().await.unwrap_or(0);
    Json(serde_json::json!({
        "version": VERSION,
        "status": "ok",
        "taskCount": task_count,
        "platform": std::env::consts::OS,
    }))
}

async fn register(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, ApiError> {
    if !is_localhost(&addr) {
        return Err(ApiError::Forbidden("Localhost only".into()));
    }

    let registration = register_app_with_approval(
        &state.app_handle,
        &state.db,
        &body.app_id,
        &body.app_name,
        &body.origin,
        &state.hmac_secret,
    )
    .await
    .map_err(|e| {
        if e.contains("denied") {
            ApiError::Forbidden(e)
        } else {
            ApiError::Internal(e)
        }
    })?;

    state.schedule_tray_refresh();

    Ok(Json(RegisterResponse {
        token: registration.token,
        app_id: registration.app_id,
    }))
}

async fn list_tasks(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let auth = authenticate(&state, &addr, &headers).await?;
    check_rate_limit(&state, &auth.app_id)?;

    let tasks = state
        .db
        .list_tasks_for_app(&auth.app_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let app = state.db.get_app_by_id(&auth.app_id).await.ok().flatten();
    let app_name = app.as_ref().map(|a| a.app_name.as_str());

    let json: Vec<_> = tasks
        .iter()
        .map(|t| task_row_to_json(t, app_name))
        .collect();
    Ok(Json(json))
}

async fn create_task(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<CreateTaskRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let auth = authenticate(&state, &addr, &headers).await?;
    check_rate_limit(&state, &auth.app_id)?;

    let (px, py) = body
        .position
        .map(|p| (Some(p.x), Some(p.y)))
        .unwrap_or((None, None));

    let metadata = body
        .metadata
        .as_ref()
        .map(|m| serde_json::to_string(m).unwrap_or_default());

    let new_task = NewTask {
        source_app_id: auth.app_id.clone(),
        title: body.title,
        body: body.body,
        priority: body.priority.unwrap_or(0),
        color: body.color.unwrap_or_else(|| "#FFE066".to_string()),
        position_x: px,
        position_y: py,
        monitor_id: body.monitor_id,
        remind_at: body.remind_at,
        metadata,
    };

    let task = state
        .db
        .create_task(&new_task)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    window_manager::create_sticky_window_async(&task, &state.app_handle)
        .await
        .map_err(|e| ApiError::Internal(e))?;

    let app = state.db.get_app_by_id(&auth.app_id).await.ok().flatten();
    let json = task_row_to_json(&task, app.as_ref().map(|a| a.app_name.as_str()));

    state.broadcast_app_event(
        &auth.app_id,
        serde_json::json!({
            "type": "task:created",
            "task": json.clone(),
        }),
    );

    Ok(Json(json))
}

async fn get_task(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let auth = authenticate(&state, &addr, &headers).await?;
    check_rate_limit(&state, &auth.app_id)?;

    let task = state
        .db
        .get_task(&id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Task not found".into()))?;

    if task.source_app_id != auth.app_id {
        return Err(ApiError::NotFound("Task not found".into()));
    }

    let app = state.db.get_app_by_id(&auth.app_id).await.ok().flatten();
    Ok(Json(task_row_to_json(
        &task,
        app.as_ref().map(|a| a.app_name.as_str()),
    )))
}

async fn update_task(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<UpdateTaskRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let auth = authenticate(&state, &addr, &headers).await?;
    check_rate_limit(&state, &auth.app_id)?;

    let existing = state
        .db
        .get_task(&id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Task not found".into()))?;

    if existing.source_app_id != auth.app_id {
        return Err(ApiError::NotFound("Task not found".into()));
    }

    let metadata = body
        .metadata
        .as_ref()
        .map(|m| serde_json::to_string(m).unwrap_or_default());

    let update = TaskUpdate {
        title: body.title,
        body: body.body,
        priority: body.priority,
        color: body.color,
        remind_at: body.remind_at,
        metadata,
        ..Default::default()
    };

    let task = state
        .db
        .update_task(&id, &update)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Task not found".into()))?;

    window_manager::update_sticky_content(&id, &task, &state.app_handle);

    let app = state.db.get_app_by_id(&auth.app_id).await.ok().flatten();
    let json = task_row_to_json(&task, app.as_ref().map(|a| a.app_name.as_str()));

    state.broadcast_app_event(
        &auth.app_id,
        serde_json::json!({
            "type": "task:updated",
            "task": json.clone(),
        }),
    );

    Ok(Json(json))
}

async fn delete_task(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let auth = authenticate(&state, &addr, &headers).await?;
    check_rate_limit(&state, &auth.app_id)?;

    let existing = state
        .db
        .get_task(&id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Task not found".into()))?;

    if existing.source_app_id != auth.app_id {
        return Err(ApiError::NotFound("Task not found".into()));
    }

    let app = state.db.get_app_by_id(&auth.app_id).await.ok().flatten();
    let task_json = task_row_to_json(&existing, app.as_ref().map(|a| a.app_name.as_str()));

    state
        .db
        .delete_task(&id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    window_manager::destroy_sticky_window(&id, &state.app_handle);

    state.broadcast_app_event(
        &auth.app_id,
        serde_json::json!({
            "type": "task:dismissed",
            "taskId": id,
            "task": task_json,
        }),
    );

    Ok(StatusCode::NO_CONTENT)
}

async fn complete_task(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let auth = authenticate(&state, &addr, &headers).await?;
    check_rate_limit(&state, &auth.app_id)?;

    let existing = state
        .db
        .get_task(&id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Task not found".into()))?;

    if existing.source_app_id != auth.app_id {
        return Err(ApiError::NotFound("Task not found".into()));
    }

    let update = TaskUpdate {
        status: Some("completed".to_string()),
        ..Default::default()
    };

    let task = state
        .db
        .update_task(&id, &update)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Task not found".into()))?;

    window_manager::destroy_sticky_window(&id, &state.app_handle);

    let app = state.db.get_app_by_id(&auth.app_id).await.ok().flatten();
    let json = task_row_to_json(&task, app.as_ref().map(|a| a.app_name.as_str()));

    state.broadcast_app_event(
        &auth.app_id,
        serde_json::json!({
            "type": "task:completed",
            "taskId": id,
            "task": json.clone(),
        }),
    );

    Ok(Json(json))
}

#[derive(Deserialize)]
pub struct RemindRequest {
    pub message: Option<String>,
}

async fn remind_task(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<RemindRequest>,
) -> Result<StatusCode, ApiError> {
    let auth = authenticate(&state, &addr, &headers).await?;
    check_rate_limit(&state, &auth.app_id)?;

    let task = state
        .db
        .get_task(&id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Task not found".into()))?;

    if task.source_app_id != auth.app_id {
        return Err(ApiError::NotFound("Task not found".into()));
    }

    use tauri_plugin_notification::NotificationExt;
    state
        .app_handle
        .notification()
        .builder()
        .title(&task.title)
        .body(
            body.message
                .as_deref()
                .or(task.body.as_deref())
                .unwrap_or("Reminder"),
        )
        .show()
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let app = state.db.get_app_by_id(&auth.app_id).await.ok().flatten();
    let json = task_row_to_json(&task, app.as_ref().map(|a| a.app_name.as_str()));

    state.broadcast_app_event(
        &auth.app_id,
        serde_json::json!({
            "type": "task:reminder",
            "task": json,
        }),
    );

    Ok(StatusCode::NO_CONTENT)
}

async fn snooze_task(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<SnoozeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let auth = authenticate(&state, &addr, &headers).await?;
    check_rate_limit(&state, &auth.app_id)?;

    let update = TaskUpdate {
        status: Some("snoozed".to_string()),
        remind_at: Some(body.until),
        ..Default::default()
    };

    let task = state
        .db
        .update_task(&id, &update)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Task not found".into()))?;

    window_manager::destroy_sticky_window(&id, &state.app_handle);

    let app = state.db.get_app_by_id(&auth.app_id).await.ok().flatten();
    let json = task_row_to_json(&task, app.as_ref().map(|a| a.app_name.as_str()));

    state.broadcast_app_event(
        &auth.app_id,
        serde_json::json!({
            "type": "task:updated",
            "task": json.clone(),
        }),
    );

    Ok(Json(json))
}

async fn authenticate(
    state: &AppState,
    addr: &std::net::SocketAddr,
    headers: &HeaderMap,
) -> Result<AuthContext, ApiError> {
    authenticate_request(state, addr, headers)
        .await
        .map_err(|e| ApiError::Unauthorized(e))
}

fn check_rate_limit(state: &AppState, app_id: &str) -> Result<(), ApiError> {
    if state.rate_limiter.check(app_id) {
        Ok(())
    } else {
        Err(ApiError::RateLimited)
    }
}

fn is_localhost(addr: &std::net::SocketAddr) -> bool {
    match addr.ip() {
        std::net::IpAddr::V4(v4) => v4.is_loopback(),
        std::net::IpAddr::V6(v6) => v6.is_loopback(),
    }
}

pub enum ApiError {
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    RateLimited,
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg).into_response(),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg).into_response(),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),
            ApiError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                [("Retry-After", "60")],
                "Rate limit exceeded",
            )
                .into_response(),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response(),
        }
    }
}
