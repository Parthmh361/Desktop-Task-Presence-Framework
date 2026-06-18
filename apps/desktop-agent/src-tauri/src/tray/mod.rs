use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::db::Database;
use crate::window_manager;

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_all = MenuItem::with_id(app, "show_all", "Show All Stickies", true, None::<&str>)?;
    let hide_all = MenuItem::with_id(app, "hide_all", "Hide All Stickies", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_all, &hide_all, &separator, &quit])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("DTPF Agent")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show_all" => window_manager::show_all_stickies(app),
            "hide_all" => window_manager::hide_all_stickies(app),
            "quit" => app.exit(0),
            _ => {}
        })
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

pub async fn update_tray_tooltip(app: &AppHandle, db: &Database) {
    let count = db.count_active_tasks().await.unwrap_or(0);
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(format!("DTPF Agent — {} active tasks", count)));
    }
}
