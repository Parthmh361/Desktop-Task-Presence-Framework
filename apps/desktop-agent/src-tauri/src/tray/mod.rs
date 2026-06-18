use tauri_plugin_updater::UpdaterExt;
use std::sync::Arc;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

use crate::db::TaskRow;
use crate::window_manager;
use crate::AppState;

pub const TRAY_ID: &str = "dtpf-tray";

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_static_menu(app)?;
    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("DTPF Agent")
        .menu(&menu)
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn build_static_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, tauri::Error> {
    let show_all = MenuItem::with_id(app, "show_all", "Show All Stickies", true, None::<&str>)?;
    let hide_all = MenuItem::with_id(app, "hide_all", "Hide All Stickies", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let preferences =
        MenuItem::with_id(app, "preferences", "Preferences", true, None::<&str>)?;
    let check_updates = MenuItem::with_id(
        app,
        "check_updates",
        "Check for Updates",
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let tasks_sub = Submenu::with_id_and_items(
        app,
        "active_tasks_sub",
        "Active Tasks (0)",
        true,
        &[],
    )?;
    let apps_sub = Submenu::with_id_and_items(
        app,
        "connected_apps_sub",
        "Connected Apps (0)",
        true,
        &[],
    )?;

    Menu::with_items(
        app,
        &[
            &tasks_sub,
            &apps_sub,
            &separator,
            &show_all,
            &hide_all,
            &separator,
            &preferences,
            &check_updates,
            &separator,
            &quit,
        ],
    )
}

pub async fn refresh_tray(state: &Arc<AppState>) {
    let db = state.db.clone();
    let app = state.app_handle.clone();

    let tasks = db.list_active_tasks().await.unwrap_or_default();
    let apps = db.list_registered_apps().await.unwrap_or_default();
    let count = tasks.len();

    let app_for_main = app.clone();
    let tasks_for_main = tasks.clone();
    let apps_for_main = apps.clone();

    let _ = app.run_on_main_thread(move || {
        if let Err(e) = rebuild_tray_menu(&app_for_main, &tasks_for_main, &apps_for_main) {
            tracing::warn!("Failed to refresh tray menu: {}", e);
        }
        update_tray_tooltip_sync(&app_for_main, count);
    });
}

fn rebuild_tray_menu(
    app: &AppHandle,
    tasks: &[TaskRow],
    apps: &[crate::db::AppRegistration],
) -> Result<(), Box<dyn std::error::Error>> {
    let show_all = MenuItem::with_id(app, "show_all", "Show All Stickies", true, None::<&str>)?;
    let hide_all = MenuItem::with_id(app, "hide_all", "Hide All Stickies", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let preferences =
        MenuItem::with_id(app, "preferences", "Preferences", true, None::<&str>)?;
    let check_updates = MenuItem::with_id(
        app,
        "check_updates",
        "Check for Updates",
        false,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let mut task_items: Vec<MenuItem<tauri::Wry>> = Vec::new();
    if tasks.is_empty() {
        task_items.push(MenuItem::with_id(
            app,
            "no_tasks",
            "(no active tasks)",
            false,
            None::<&str>,
        )?);
    } else {
        for task in tasks {
            let label = format!("{} [{}]", truncate(&task.title, 40), task.source_app_id);
            task_items.push(MenuItem::with_id(
                app,
                format!("task_{}", task.id),
                label,
                true,
                None::<&str>,
            )?);
        }
    }

    let task_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = task_items
        .iter()
        .map(|item| item as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
        .collect();

    let tasks_sub = Submenu::with_id_and_items(
        app,
        "active_tasks_sub",
        &format!("Active Tasks ({})", tasks.len()),
        true,
        &task_refs,
    )?;

    let mut app_items: Vec<MenuItem<tauri::Wry>> = Vec::new();
    if apps.is_empty() {
        app_items.push(MenuItem::with_id(
            app,
            "no_apps",
            "(no connected apps)",
            false,
            None::<&str>,
        )?);
    } else {
        for reg in apps {
            app_items.push(MenuItem::with_id(
                app,
                format!("app_{}", reg.app_id.replace('.', "_")),
                &reg.app_name,
                false,
                None::<&str>,
            )?);
        }
    }

    let app_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> = app_items
        .iter()
        .map(|item| item as &dyn tauri::menu::IsMenuItem<tauri::Wry>)
        .collect();

    let apps_sub = Submenu::with_id_and_items(
        app,
        "connected_apps_sub",
        &format!("Connected Apps ({})", apps.len()),
        true,
        &app_refs,
    )?;

    let menu = Menu::with_items(
        app,
        &[
            &tasks_sub,
            &apps_sub,
            &separator,
            &show_all,
            &hide_all,
            &separator,
            &preferences,
            &check_updates,
            &separator,
            &quit,
        ],
    )?;

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(menu))?;
    }

    Ok(())
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id.as_ref() {
        "show_all" => window_manager::show_all_stickies(app),
        "hide_all" => window_manager::hide_all_stickies(app),
        "preferences" => {
            app.dialog()
                .message("Preferences UI coming in a future release.")
                .title("DTPF Preferences")
                .kind(MessageDialogKind::Info)
                .show(|_| {});
        }
        "check_updates" => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                match app.updater() {
                    Ok(updater) => match updater.check().await {
                        Ok(Some(update)) => {
                            update.download_and_install().await?;
                        }
                        Ok(None) => { app.dialog()
                .message("Auto-updates will be enabled after the first signed release.")
                .title("Check for Updates")
                .kind(MessageDialogKind::Info)
                .show(|_| {}); }
                        Err(e) => { app.dialog()
                .message("Failed to check for updates: {}", e)
                .title("Check for Updates")
                .kind(MessageDialogKind::Error)
                .show(|_| {}); }
                    },
                    Err(e) => { app.dialog()
                .message("Updater not configured: {}", e)
                .title("Check for Updates")
                .kind(MessageDialogKind::Error)
                .show(|_| {}); }
                }
            });
        }
        "quit" => app.exit(0),
        id if id.starts_with("task_") => {
            let task_id = id.strip_prefix("task_").unwrap_or(id);
            window_manager::focus_sticky_window(task_id, app);
        }
        _ => {}
    }
}

fn update_tray_tooltip_sync(app: &AppHandle, count: usize) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(format!("DTPF Agent — {} active tasks", count)));
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max.saturating_sub(1)).collect::<String>())
    }
}
