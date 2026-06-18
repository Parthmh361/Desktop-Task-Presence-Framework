use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tower_http::cors::{Any, CorsLayer};

use crate::db::{AppRegistration, Database, NewAppRegistration};
use crate::server::routes::router;
use crate::AppState;

type HmacSha256 = Hmac<Sha256>;

const AGENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_PORT: u16 = 7842;

pub mod auth;
pub mod routes;
pub mod websocket;

pub async fn start_server(state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = router(state.clone()).layer(cors);
    let addr = SocketAddr::from(([127, 0, 0, 1], DEFAULT_PORT));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("DTPF agent listening on http://{}", addr);

    write_lock_file(DEFAULT_PORT)?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

fn write_lock_file(port: u16) -> Result<(), std::io::Error> {
    let dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".dtpf");
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("agent.lock"), format!("http://127.0.0.1:{}", port))?;
    Ok(())
}

pub fn get_or_create_secret() -> String {
    let path = dirs::home_dir()
        .unwrap_or_default()
        .join(".dtpf")
        .join("secret.key");

    if path.exists() {
        if let Ok(secret) = std::fs::read_to_string(&path) {
            if !secret.trim().is_empty() {
                return secret.trim().to_string();
            }
        }
    }

    let secret: String = (0..32)
        .map(|_| format!("{:02x}", rand::random::<u8>()))
        .collect();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&path, &secret).ok();
    secret
}

pub fn generate_token(app_id: &str, origin: &str, secret: &str) -> String {
    let created_at = chrono::Utc::now().timestamp();
    let payload = format!("{}:{}:{}", app_id, origin, created_at);
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub async fn register_app_with_approval(
    app_handle: &AppHandle,
    db: &Database,
    app_id: &str,
    app_name: &str,
    origin: &str,
    secret: &str,
) -> Result<AppRegistration, String> {
    if let Some(existing) = db
        .get_app_by_id(app_id)
        .await
        .map_err(|e| e.to_string())?
    {
        return Ok(existing);
    }

    let message = format!(
        "{} ({}) wants to create desktop sticky notes on your desktop.\n\nAllow this application?",
        app_name, origin
    );

    let app_handle = app_handle.clone();
    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();

    tracing::info!(
        "Authorization requested for {} ({}) — showing approval dialog",
        app_name,
        origin
    );

    app_handle
        .dialog()
        .message(message)
        .title("DTPF Authorization")
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Allow".to_string(),
            "Deny".to_string(),
        ))
        .show(move |approved| {
            let _ = tx.send(approved);
        });

    let approved = rx.await.map_err(|e| e.to_string())?;

    if !approved {
        return Err("User denied authorization".to_string());
    }

    let token = generate_token(app_id, origin, secret);
    let registration = NewAppRegistration {
        app_id: app_id.to_string(),
        app_name: app_name.to_string(),
        origin: origin.to_string(),
        token: token.clone(),
    };

    db.register_app(&registration)
        .await
        .map_err(|e| e.to_string())?;

    db.get_app_by_id(app_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Failed to load registration".to_string())
}

pub struct RateLimiter {
    windows: DashMap<String, VecDeque<Instant>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            windows: DashMap::new(),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    pub fn check(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut entry = self.windows.entry(key.to_string()).or_default();
        entry.retain(|t| now.duration_since(*t) < self.window);
        if entry.len() >= self.max_requests {
            return false;
        }
        entry.push_back(now);
        true
    }
}

pub fn task_row_to_json(task: &crate::db::TaskRow, app_name: Option<&str>) -> serde_json::Value {
    serde_json::json!({
        "id": task.id,
        "title": task.title,
        "body": task.body,
        "status": task.status,
        "priority": task.priority,
        "color": task.color,
        "remindAt": task.remind_at.map(|t| chrono::DateTime::from_timestamp(t, 0)),
        "position": match (task.position_x, task.position_y) {
            (Some(x), Some(y)) => Some(serde_json::json!({"x": x, "y": y})),
            _ => None,
        },
        "monitorId": task.monitor_id,
        "sourceAppId": task.source_app_id,
        "sourceAppName": app_name,
        "metadata": task.metadata.as_ref().and_then(|m| serde_json::from_str::<serde_json::Value>(m).ok()),
        "createdAt": chrono::DateTime::from_timestamp(task.created_at, 0),
        "updatedAt": chrono::DateTime::from_timestamp(task.updated_at, 0),
    })
}

pub const VERSION: &str = AGENT_VERSION;

#[cfg(test)]
mod tests {
    use super::generate_token;

    #[test]
    fn generate_token_is_deterministic_for_same_second() {
        let secret = "test-secret-key-32-bytes-long!!";
        let t1 = generate_token("app1", "http://localhost:5173", secret);
        let t2 = generate_token("app1", "http://localhost:5173", secret);
        assert_eq!(t1.len(), 64);
        assert_eq!(t1, t2);
    }
}
