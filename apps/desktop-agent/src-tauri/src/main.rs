#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Always-on-top is unreliable on native Wayland; prefer X11/XWayland on Linux.
    #[cfg(target_os = "linux")]
    unsafe {
        std::env::set_var("GDK_BACKEND", "x11");
    }
    dtpf_agent_lib::run();
}
