const SERVICE: &str = "com.dtpf.agent";
const USER: &str = "hmac-secret";

/// Load or create the HMAC secret, preferring OS keychain with file fallback.
pub fn get_or_create_secret() -> String {
    if let Ok(secret) = read_keyring_secret() {
        if !secret.trim().is_empty() {
            return secret.trim().to_string();
        }
    }

    let path = crate::paths::secret_key_path();
    if path.exists() {
        if let Ok(secret) = std::fs::read_to_string(&path) {
            if !secret.trim().is_empty() {
                let trimmed = secret.trim().to_string();
                store_keyring_secret(&trimmed);
                return trimmed;
            }
        }
    }

    let secret: String = (0..32)
        .map(|_| format!("{:02x}", rand::random::<u8>()))
        .collect();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&path, &secret).ok();
    store_keyring_secret(&secret);
    secret
}

fn read_keyring_secret() -> Result<String, keyring::Error> {
    keyring::Entry::new(SERVICE, USER)?.get_password()
}

fn store_keyring_secret(secret: &str) {
    if let Ok(entry) = keyring::Entry::new(SERVICE, USER) {
        let _ = entry.set_password(secret);
    }
}
