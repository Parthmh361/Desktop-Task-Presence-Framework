use std::path::PathBuf;

/// Root directory for agent data (database, secrets, lock file).
pub fn data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        dirs::data_local_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("dtpf")
    }

    #[cfg(target_os = "macos")]
    {
        dirs::data_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("dtpf")
    }

    #[cfg(target_os = "linux")]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".dtpf")
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".dtpf")
    }
}

pub fn db_path() -> PathBuf {
    data_dir().join("tasks.db")
}

pub fn secret_key_path() -> PathBuf {
    data_dir().join("secret.key")
}

pub fn agent_lock_path() -> PathBuf {
    data_dir().join("agent.lock")
}
