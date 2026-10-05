//! Database key kept in the operating system keychain.

use super::StorageError;
use keyring_core::{Entry, Error as KeyringError};
use std::sync::Once;

const SERVICE: &str = "mx.cimiento.app";
const DB_KEY_USER: &str = "database-key";

static INIT: Once = Once::new();

fn init_store() -> Result<(), StorageError> {
    let mut result = Ok(());
    INIT.call_once(|| {
        #[cfg(windows)]
        match windows_native_keyring_store::Store::new() {
            Ok(store) => keyring_core::set_default_store(store),
            Err(e) => result = Err(StorageError::Keychain(e)),
        }
    });
    result
}

/// Returns the 256-bit database key as hex, creating and storing it on first use.
pub fn get_or_create_db_key() -> Result<String, StorageError> {
    get_or_create(SERVICE, DB_KEY_USER)
}

fn get_or_create(service: &str, user: &str) -> Result<String, StorageError> {
    init_store()?;
    let entry = Entry::new(service, user)?;
    match entry.get_secret() {
        Ok(bytes) => Ok(hex::encode(bytes)),
        Err(KeyringError::NoEntry) => {
            let mut key = [0u8; 32];
            getrandom::fill(&mut key).map_err(|e| StorageError::Random(e.to_string()))?;
            entry.set_secret(&key)?;
            Ok(hex::encode(key))
        }
        Err(e) => Err(e.into()),
    }
}

/// Keychain entry of each AI provider. The Anthropic name is the one used since Phase 2,
/// so a key saved back then is still found.
fn api_key_user(provider: &str) -> String {
    match provider {
        "anthropic" => "anthropic-api-key".to_string(),
        other => format!("{other}-api-key"),
    }
}

/// An AI provider key. It lives only in the OS keychain: never in files, never in the UI.
pub fn get_api_key(provider: &str) -> Result<Option<String>, StorageError> {
    init_store()?;
    match Entry::new(SERVICE, &api_key_user(provider))?.get_secret() {
        Ok(bytes) => Ok(String::from_utf8(bytes).ok().filter(|k| !k.is_empty())),
        Err(KeyringError::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn set_api_key(provider: &str, key: &str) -> Result<(), StorageError> {
    init_store()?;
    Entry::new(SERVICE, &api_key_user(provider))?.set_secret(key.trim().as_bytes())?;
    Ok(())
}

pub fn delete_api_key(provider: &str) -> Result<(), StorageError> {
    init_store()?;
    match Entry::new(SERVICE, &api_key_user(provider))?.delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn key_is_created_once_and_then_reused() {
        let user = "test-database-key";
        let first = get_or_create("mx.cimiento.app.test", user).unwrap();
        let second = get_or_create("mx.cimiento.app.test", user).unwrap();
        assert_eq!(first.len(), 64);
        assert_eq!(first, second);
        Entry::new("mx.cimiento.app.test", user)
            .unwrap()
            .delete_credential()
            .unwrap();
    }
}
