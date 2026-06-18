#[cfg(target_os = "linux")]
pub fn register_startup() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let service_dir = dirs::home_dir()
        .ok_or_else(|| "No home directory".to_string())?
        .join(".config/systemd/user");

    std::fs::create_dir_all(&service_dir).map_err(|e| e.to_string())?;

    let service_content = format!(
        "[Unit]\nDescription=DTPF Desktop Agent\n\n[Service]\nExecStart={}\nRestart=on-failure\n\n[Install]\nWantedBy=default.target\n",
        exe.display()
    );

    let service_path = service_dir.join("dtpf-agent.service");
    std::fs::write(&service_path, service_content).map_err(|e| e.to_string())?;

    std::process::Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()
        .ok();

    std::process::Command::new("systemctl")
        .args(["--user", "enable", "dtpf-agent.service"])
        .status()
        .ok();

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn register_startup() -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run = hkcu
        .open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            KEY_SET_VALUE,
        )
        .map_err(|e| e.to_string())?;
    run.set_value("DTPF", &exe.to_str().ok_or("Invalid exe path")?)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn register_startup() -> Result<(), String> {
    Ok(())
}

pub fn setup_autostart() {
    if let Err(e) = register_startup() {
        tracing::warn!("Failed to register auto-start: {}", e);
    }
}
