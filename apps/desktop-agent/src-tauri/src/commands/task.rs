use std::sync::Arc;

use tauri::State;

use crate::db::TaskUpdate;
use crate::server::task_row_to_json;
use crate::window_manager;
use crate::AppState;

#[tauri::command]
pub async fn get_task(
    task_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<serde_json::Value, String> {
    let task = state
        .db
        .get_task(&task_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Task not found".to_string())?;

    let app = state
        .db
        .get_app_by_id(&task.source_app_id)
        .await
        .ok()
        .flatten();

    Ok(task_row_to_json(
        &task,
        app.as_ref().map(|a| a.app_name.as_str()),
    ))
}

#[tauri::command]
pub async fn complete_task_from_ui(
    task_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let update = TaskUpdate {
        status: Some("completed".to_string()),
        ..Default::default()
    };

    let task = state
        .db
        .update_task(&task_id, &update)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Task not found".to_string())?;

    window_manager::destroy_sticky_window(&task_id, &state.app_handle);

    let app = state
        .db
        .get_app_by_id(&task.source_app_id)
        .await
        .ok()
        .flatten();
    let json = task_row_to_json(&task, app.as_ref().map(|a| a.app_name.as_str()));

    state.broadcast_app_event(
        &task.source_app_id,
        serde_json::json!({
            "type": "task:completed",
            "taskId": task_id,
            "task": json,
        }),
    );

    Ok(())
}

#[tauri::command]
pub async fn dismiss_task_from_ui(
    task_id: String,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let task = state
        .db
        .get_task(&task_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Task not found".to_string())?;

    let app = state
        .db
        .get_app_by_id(&task.source_app_id)
        .await
        .ok()
        .flatten();
    let json = task_row_to_json(&task, app.as_ref().map(|a| a.app_name.as_str()));
    let source_app_id = task.source_app_id.clone();

    state
        .db
        .delete_task(&task_id)
        .await
        .map_err(|e| e.to_string())?;

    window_manager::destroy_sticky_window(&task_id, &state.app_handle);

    state.broadcast_app_event(
        &source_app_id,
        serde_json::json!({
            "type": "task:dismissed",
            "taskId": task_id,
            "task": json,
        }),
    );

    Ok(())
}

#[tauri::command]
pub async fn save_window_position(
    task_id: String,
    x: i32,
    y: i32,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state
        .db
        .update_position(&task_id, x, y)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
