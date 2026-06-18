use std::sync::Arc;

use tauri::AppHandle;
use tokio::sync::broadcast;

use crate::db::Database;
use crate::server::RateLimiter;

pub struct AppState {
    pub db: Arc<Database>,
    pub app_handle: AppHandle,
    pub event_tx: broadcast::Sender<String>,
    pub hmac_secret: String,
    pub rate_limiter: RateLimiter,
}

impl AppState {
    pub fn broadcast_event(&self, event: serde_json::Value) {
        if let Ok(msg) = serde_json::to_string(&event) {
            let _ = self.event_tx.send(msg);
        }
    }

    pub fn broadcast_app_event(&self, source_app_id: &str, mut event: serde_json::Value) {
        if let serde_json::Value::Object(ref mut map) = event {
            map.insert(
                "sourceAppId".to_string(),
                serde_json::json!(source_app_id),
            );
        }
        self.broadcast_event(event);
    }
}
