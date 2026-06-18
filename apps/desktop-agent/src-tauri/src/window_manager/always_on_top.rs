use std::time::Duration;

use tauri::{AppHandle, Manager, WebviewWindow};

#[cfg(target_os = "linux")]
use gtk::prelude::*;

#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
};

/// Must run on the Tauri/GTK main thread only.
fn pin_sticky_window_sync(window: &WebviewWindow) {
    let _ = window.set_always_on_top(true);

    #[cfg(target_os = "linux")]
    {
        if let Ok(gtk_window) = window.gtk_window() {
            gtk_window.set_keep_above(true);
            gtk_window.set_skip_taskbar_hint(true);
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(hwnd) = window.hwnd() {
            unsafe {
                let _ = SetWindowPos(
                    hwnd,
                    Some(HWND_TOPMOST),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
                );
            }
        }
    }
}

/// Schedule always-on-top pinning on the main thread (safe from any thread).
pub fn schedule_pin_sticky_window(window: &WebviewWindow) {
    let label = window.label().to_string();
    let app = window.app_handle().clone();
    let app_in_closure = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(window) = app_in_closure.get_webview_window(&label) {
            pin_sticky_window_sync(&window);
        }
    });
}

pub fn attach_sticky_pin_watchers(window: WebviewWindow) {
    pin_sticky_window_sync(&window);

    let label = window.label().to_string();
    let event_label = label.clone();
    let loop_label = label;
    let app = window.app_handle().clone();
    let event_app = app.clone();

    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(focused) = event {
            if !focused {
                if let Some(win) = event_app.get_webview_window(&event_label) {
                    pin_sticky_window_sync(&win);
                }
            }
        }
    });

    start_pin_loop(app, loop_label);
}

fn start_pin_loop(app: AppHandle, label: String) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(3)).await;
            if app.get_webview_window(&label).is_none() {
                break;
            }
            let app_for_main = app.clone();
            let label_for_main = label.clone();
            let app_in_closure = app_for_main.clone();
            if app_for_main
                .run_on_main_thread(move || {
                    if let Some(window) = app_in_closure.get_webview_window(&label_for_main) {
                        pin_sticky_window_sync(&window);
                    }
                })
                .is_err()
            {
                break;
            }
        }
    });
}
