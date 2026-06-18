use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::db::{Database, TaskRow};

mod always_on_top;
use always_on_top::{attach_sticky_pin_watchers, schedule_pin_sticky_window};

/// Create a sticky window. Must be called on the Tauri main thread.
pub fn create_sticky_window(task: &TaskRow, app: &AppHandle) -> Result<(), String> {
    let label = window_label(&task.id);

    if app.get_webview_window(&label).is_some() {
        return Ok(());
    }

    let (x, y) = default_position(task, app);

    let url = format!("index.html?taskId={}", task.id);
    let builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url.into()))
        .title("")
        .inner_size(280.0, 200.0)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(true)
        .visible(true)
        .position(x, y);

    #[cfg(not(target_os = "macos"))]
    let builder = builder.transparent(true);

    let window = builder
        .build()
        .map_err(|e| e.to_string())?;

    attach_sticky_pin_watchers(window);
    Ok(())
}

pub async fn create_sticky_window_async(task: &TaskRow, app: &AppHandle) -> Result<(), String> {
    let task = task.clone();
    let app = app.clone();
    let app_for_main = app.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();

    app.run_on_main_thread(move || {
        let result = create_sticky_window(&task, &app_for_main);
        let _ = tx.send(result);
    })
    .map_err(|e| e.to_string())?;

    rx.await.map_err(|e| e.to_string())?
}

pub fn destroy_sticky_window(task_id: &str, app: &AppHandle) {
    if let Some(window) = app.get_webview_window(&window_label(task_id)) {
        window.close().ok();
    }
}

pub async fn restore_all_windows(db: &Database, app: &AppHandle) -> Result<(), String> {
    let tasks = db.list_active_tasks().await.map_err(|e| e.to_string())?;
    let app = app.clone();
    let app_for_main = app.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();

    app.run_on_main_thread(move || {
        let result = (|| {
            for (index, task) in tasks.iter().enumerate() {
                let mut task = task.clone();
                if !monitor_exists(&app_for_main, &task.monitor_id)
                    || task.position_x.is_none()
                    || task.position_y.is_none()
                {
                    if let Some(monitor) = app_for_main.primary_monitor().ok().flatten() {
                        let pos = monitor.position();
                        task.position_x = Some(pos.x as i64 + 100 + (index as i64 * 20));
                        task.position_y = Some(pos.y as i64 + 100 + (index as i64 * 20));
                    } else {
                        task.position_x = Some(100 + (index as i64 * 20));
                        task.position_y = Some(100 + (index as i64 * 20));
                    }
                    task.monitor_id = app_for_main
                        .primary_monitor()
                        .ok()
                        .flatten()
                        .and_then(|m| m.name().map(|n| n.to_string()));
                }
                create_sticky_window(&task, &app_for_main)?;
            }
            Ok(())
        })();
        let _ = tx.send(result);
    })
    .map_err(|e| e.to_string())?;

    rx.await.map_err(|e| e.to_string())?
}

pub fn update_sticky_content(task_id: &str, task: &TaskRow, app: &AppHandle) {
    let label = window_label(task_id);
    if let Some(window) = app.get_webview_window(&label) {
        let payload = serde_json::json!({
            "id": task.id,
            "title": task.title,
            "body": task.body,
            "status": task.status,
            "priority": task.priority,
            "color": task.color,
            "remindAt": task.remind_at.map(|t| chrono::DateTime::from_timestamp(t, 0)),
        });
        window.emit("task:updated", payload).ok();
        schedule_pin_sticky_window(&window);
    }
}

pub fn show_all_stickies(app: &AppHandle) {
    for window in app.webview_windows().values() {
        if window.label().starts_with("sticky-") {
            window.show().ok();
            schedule_pin_sticky_window(&window);
        }
    }
}

pub fn hide_all_stickies(app: &AppHandle) {
    for window in app.webview_windows().values() {
        if window.label().starts_with("sticky-") {
            window.hide().ok();
        }
    }
}

pub fn focus_sticky_window(task_id: &str, app: &AppHandle) {
    let label = window_label(task_id);
    if let Some(window) = app.get_webview_window(&label) {
        window.show().ok();
        window.set_focus().ok();
        schedule_pin_sticky_window(&window);
    }
}

pub fn monitor_id_at_position(app: &AppHandle, x: i32, y: i32) -> Option<String> {
    app.available_monitors().ok()?.into_iter().find_map(|m| {
        let pos = m.position();
        let size = m.size();
        let in_bounds = x >= pos.x
            && x <= pos.x + size.width as i32
            && y >= pos.y
            && y <= pos.y + size.height as i32;
        if in_bounds {
            m.name().map(|n| n.to_string())
        } else {
            None
        }
    })
}

fn window_label(task_id: &str) -> String {
    format!("sticky-{}", task_id)
}

fn default_position(task: &TaskRow, app: &AppHandle) -> (f64, f64) {
    if let (Some(x), Some(y)) = (task.position_x, task.position_y) {
        if monitor_exists(app, &task.monitor_id) {
            return (x as f64, y as f64);
        }
    }

    if let Some(monitor) = app.primary_monitor().ok().flatten() {
        let pos = monitor.position();
        return (pos.x as f64 + 100.0, pos.y as f64 + 100.0);
    }

    (100.0, 100.0)
}

fn monitor_exists(app: &AppHandle, monitor_id: &Option<String>) -> bool {
    let Some(id) = monitor_id else {
        return true;
    };
    app.available_monitors()
        .ok()
        .map(|monitors| {
            monitors.iter().any(|m| {
                m.name()
                    .map(|n| n.as_str() == id.as_str())
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}
