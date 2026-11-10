use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use serde_json::Value;
use std::path::Path;

use blazers_core::models::Secret;

const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

pub const EVENT_SECRET_CREATE: &str = "secret.create";
pub const EVENT_SECRET_UPDATE: &str = "secret.update";
pub const EVENT_SECRET_DELETE: &str = "secret.delete";

pub fn get_or_create_secrets_key(data_dir: &Path) -> Result<Vec<u8>, String> {
    if let Ok(env_key) = std::env::var("BLAZERS_SECRETS_KEY") {
        return hex::decode(&env_key).map_err(|e| format!("decode env key: {e}"));
    }
    let key_path = data_dir.join("secrets.key");
    if let Ok(data) = std::fs::read_to_string(&key_path) {
        return hex::decode(data.trim()).map_err(|e| format!("decode key file: {e}"));
    }
    let mut key = vec![0u8; KEY_LEN];
    rand::fill(&mut key);
    let hex_key = hex::encode(&key);
    std::fs::write(&key_path, &hex_key).map_err(|e| format!("write key file: {e}"))?;
    Ok(key)
}

pub fn encrypt_secret(key: &[u8], plaintext: &str) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("new cipher: {e}"))?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::fill(&mut nonce_bytes);
    let nonce = Nonce::try_from(&nonce_bytes[..]).map_err(|e| format!("nonce: {e}"))?;
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| format!("encrypt: {e}"))?;
    Ok(hex::encode(nonce_bytes) + ":" + &hex::encode(ciphertext))
}

pub fn decrypt_secret(key: &[u8], encrypted: &str) -> Result<String, String> {
    let parts: Vec<&str> = encrypted.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err("invalid encrypted format".into());
    }
    let nonce_bytes = hex::decode(parts[0]).map_err(|e| format!("decode nonce: {e}"))?;
    let ciphertext = hex::decode(parts[1]).map_err(|e| format!("decode ciphertext: {e}"))?;
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("new cipher: {e}"))?;
    let nonce =
        Nonce::try_from(nonce_bytes.as_slice()).map_err(|_| "invalid nonce length".to_string())?;
    let plaintext = cipher
        .decrypt(&nonce, ciphertext.as_slice())
        .map_err(|e| format!("decrypt: {e}"))?;
    String::from_utf8(plaintext).map_err(|e| format!("utf8: {e}"))
}

pub fn webhook_secret_data(secret: &Secret) -> Value {
    serde_json::json!({
        "secret": secret,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn encrypt_decrypt_round_trip() {
        let key = [0u8; 32];
        let plaintext = "my-secret-value-123";
        let encrypted = encrypt_secret(&key, plaintext).unwrap();
        let decrypted = decrypt_secret(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_with_wrong_key_returns_error() {
        let key = [0u8; 32];
        let wrong_key = [1u8; 32];
        let encrypted = encrypt_secret(&key, "secret-data").unwrap();
        let result = decrypt_secret(&wrong_key, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_invalid_format_returns_error() {
        let key = [0u8; 32];
        let result = decrypt_secret(&key, "invalid-format-no-colon");
        assert_eq!(result.unwrap_err(), "invalid encrypted format");
    }

    #[test]
    fn webhook_secret_data_correct_json() {
        let secret = Secret {
            id: "sec-1".into(),
            name: "my-secret".into(),
            value: "encrypted-value".into(),
            scope: "default".into(),
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        };
        let data = webhook_secret_data(&secret);
        assert_eq!(data["secret"]["id"], "sec-1");
        assert_eq!(data["secret"]["name"], "my-secret");
        assert_eq!(data["secret"]["value"], "encrypted-value");
    }

    #[test]
    fn event_names_are_non_empty() {
        assert!(!EVENT_SECRET_CREATE.is_empty());
        assert!(!EVENT_SECRET_UPDATE.is_empty());
        assert!(!EVENT_SECRET_DELETE.is_empty());
    }

    #[test]
    fn get_or_create_secrets_key_from_env_var() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempfile::TempDir::new().unwrap();
        let key_bytes = [0xABu8; 32];
        let hex_key = hex::encode(key_bytes);

        unsafe { std::env::set_var("BLAZERS_SECRETS_KEY", &hex_key) };
        let result = get_or_create_secrets_key(dir.path());
        unsafe { std::env::remove_var("BLAZERS_SECRETS_KEY") };

        let key = result.unwrap();
        assert_eq!(key, key_bytes);
    }

    #[test]
    fn get_or_create_secrets_key_from_existing_file() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempfile::TempDir::new().unwrap();
        let key_bytes = [0xCDu8; 32];
        let hex_key = hex::encode(key_bytes);
        let key_path = dir.path().join("secrets.key");
        std::fs::write(&key_path, &hex_key).unwrap();

        unsafe { std::env::remove_var("BLAZERS_SECRETS_KEY") };
        let result = get_or_create_secrets_key(dir.path());

        let key = result.unwrap();
        assert_eq!(key, key_bytes);
    }

    #[test]
    fn get_or_create_secrets_key_creates_new_file() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempfile::TempDir::new().unwrap();
        let key_path = dir.path().join("secrets.key");

        unsafe { std::env::remove_var("BLAZERS_SECRETS_KEY") };
        let result = get_or_create_secrets_key(dir.path());

        let key = result.unwrap();
        assert_eq!(key.len(), 32);
        assert!(key_path.exists());
        let saved_hex = std::fs::read_to_string(&key_path).unwrap();
        assert_eq!(hex::decode(saved_hex.trim()).unwrap(), key);
    }

    #[test]
    fn get_or_create_secrets_key_rejects_invalid_env_var() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempfile::TempDir::new().unwrap();

        unsafe { std::env::set_var("BLAZERS_SECRETS_KEY", "not-valid-hex") };
        let result = get_or_create_secrets_key(dir.path());
        unsafe { std::env::remove_var("BLAZERS_SECRETS_KEY") };

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("decode env key"));
    }

    #[test]
    fn get_or_create_secrets_key_rejects_invalid_file_content() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempfile::TempDir::new().unwrap();
        let key_path = dir.path().join("secrets.key");
        std::fs::write(&key_path, "not-valid-hex").unwrap();

        unsafe { std::env::remove_var("BLAZERS_SECRETS_KEY") };
        let result = get_or_create_secrets_key(dir.path());

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("decode key file"));
    }

    #[test]
    fn get_or_create_secrets_key_write_failure() {
        let _lock = ENV_LOCK.lock().unwrap();
        let dir = tempfile::TempDir::new().unwrap();
        let perm = std::fs::Permissions::from_mode(0o444);
        std::fs::set_permissions(dir.path(), perm).unwrap();
        unsafe { std::env::remove_var("BLAZERS_SECRETS_KEY") };
        let result = get_or_create_secrets_key(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn encrypt_secret_wrong_key_length() {
        let short_key = [0u8; 16];
        let result = encrypt_secret(&short_key, "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("new cipher"));
    }

    #[test]
    fn decrypt_secret_invalid_hex_nonce() {
        let key = [0u8; 32];
        let result = decrypt_secret(&key, "ZZ:abcdef");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("decode nonce"));
    }

    #[test]
    fn decrypt_secret_invalid_hex_ciphertext() {
        let key = [0u8; 32];
        let result = decrypt_secret(&key, "abcdef123456:ZZZ");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("decode ciphertext"));
    }

    #[test]
    fn decrypt_secret_invalid_nonce_length() {
        let key = [0u8; 32];
        let result = decrypt_secret(&key, "ab:abcdef");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "invalid nonce length");
    }
}
