mod commands;
mod db;
mod paths;
mod secrets;
mod server;
mod startup;
mod state;
mod sync;
mod tray;
mod window_manager;

use std::sync::Arc;

use tauri::Manager;
use tokio::sync::broadcast;

pub use state::AppState;

fn init_database() -> Arc<db::Database> {
    std::thread::spawn(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build init runtime")
            .block_on(db::Database::new())
    })
    .join()
    .expect("Database init thread panicked")
    .expect("Failed to initialize database")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            let db = init_database();
            let secret = secrets::get_or_create_secret();
            let (event_tx, _) = broadcast::channel(256);

            let state = Arc::new(AppState {
                db: db.clone(),
                app_handle: handle.clone(),
                event_tx,
                hmac_secret: secret,
                rate_limiter: server::RateLimiter::new(100, 60),
            });

            app.manage(state.clone());

            let server_state = state.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = server::start_server(server_state).await {
                    tracing::error!("HTTP server error: {}", e);
                }
            });

            spawn_reminder_task(state.clone());
            startup::setup_autostart();

            let restore_handle = handle.clone();
            let state_for_tray = state.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = window_manager::restore_all_windows(&db, &restore_handle).await {
                    tracing::warn!("Failed to restore windows: {}", e);
                }
                state_for_tray.schedule_tray_refresh();
            });

            tray::setup_tray(&handle).expect("Failed to setup system tray");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::task::get_task,
            commands::task::complete_task_from_ui,
            commands::task::dismiss_task_from_ui,
            commands::task::save_window_position,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn spawn_reminder_task(state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        let mut notified: std::collections::HashSet<String> = std::collections::HashSet::new();
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            let now = chrono::Utc::now().timestamp();
            if let Ok(tasks) = state.db.tasks_due_for_reminder(now).await {
                for task in tasks {
                    if notified.contains(&task.id) {
                        continue;
                    }
                    notified.insert(task.id.clone());

                    use tauri_plugin_notification::NotificationExt;
                    state
                        .app_handle
                        .notification()
                        .builder()
                        .title(&task.title)
                        .body(task.body.as_deref().unwrap_or("Reminder"))
                        .show()
                        .ok();

                    let app = state
                        .db
                        .get_app_by_id(&task.source_app_id)
                        .await
                        .ok()
                        .flatten();
                    state.broadcast_app_event(
                        &task.source_app_id,
                        serde_json::json!({
                            "type": "task:reminder",
                            "task": server::task_row_to_json(
                                &task,
                                app.as_ref().map(|a| a.app_name.as_str()),
                            ),
                        }),
                    );
                }
            }
        }
    });
}
