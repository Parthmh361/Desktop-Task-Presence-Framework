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
    let exe_str = exe
        .to_str()
        .ok_or("Invalid exe path")?
        .to_string();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run = hkcu
        .open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            KEY_SET_VALUE,
        )
        .map_err(|e| e.to_string())?;
    run.set_value("DTPF", &exe_str)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
pub fn register_startup() -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn register_startup() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let home = dirs::home_dir().ok_or_else(|| "No home directory".to_string())?;
    let agents_dir = home.join("Library/LaunchAgents");
    std::fs::create_dir_all(&agents_dir).map_err(|e| e.to_string())?;

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.dtpf.agent</string>
  <key>ProgramArguments</key>
  <array>
    <string>{}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <false/>
</dict>
</plist>
"#,
        exe.display()
    );

    let plist_path = agents_dir.join("com.dtpf.agent.plist");
    std::fs::write(&plist_path, plist).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn setup_autostart() {
    if let Err(e) = register_startup() {
        tracing::warn!("Failed to register auto-start: {}", e);
    }
}
