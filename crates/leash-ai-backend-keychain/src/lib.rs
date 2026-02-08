use async_trait::async_trait;
use leash_ai_backend::SecretBackend;
use leash_ai_core::{Result, LeashError};
use security_framework::os::macos::keychain::SecKeychain;
use security_framework::os::macos::passwords::find_generic_password;

pub struct KeychainBackend;

#[async_trait]
impl SecretBackend for KeychainBackend {
    async fn get_secret(&self, secret_id: &str) -> Result<String> {
        let (password, _) = find_generic_password(None, "leash-ai", secret_id)
            .map_err(|e| {
                if e.code() == -25300 { // errSecItemNotFound
                    LeashError::NotFound(format!("Secret '{}' not found in Keychain", secret_id))
                } else if e.code() == -25308 { // errSecInteractionNotAllowed
                    LeashError::Backend("Keychain is locked or interaction is required".to_string())
                } else {
                    LeashError::Backend(format!("Keychain error: {}", e))
                }
            })?;

        let secret = String::from_utf8(password.to_vec())
            .map_err(|e| LeashError::Internal(format!("Invalid UTF-8 in secret: {}", e)))?;

        Ok(secret)
    }

    async fn store_secret(&self, secret_id: &str, value: &str) -> Result<()> {
        let keychain = SecKeychain::default()
            .map_err(|e| LeashError::Backend(format!("Failed to open default keychain: {}", e)))?;

        // Try to delete if exists to perform an "update"
        if let Ok((_, item)) = find_generic_password(None, "leash-ai", secret_id) {
            // SecKeychainItem deletion
            let _ = item.delete();
        }

        keychain.add_generic_password("leash-ai", secret_id, value.as_bytes())
            .map_err(|e| LeashError::Backend(format!("Failed to store secret in keychain: {}", e)))?;

        Ok(())
    }

    async fn is_locked(&self) -> Result<bool> {
        match find_generic_password(None, "leash-ai", "dummy-locked-check") {
            Ok(_) => Ok(false),
            Err(e) if e.code() == -25308 => Ok(true),
            _ => Ok(false),
        }
    }

    async fn unlock(&self, password: Option<&str>) -> Result<()> {
        let mut keychain = SecKeychain::default()
            .map_err(|e| LeashError::Backend(format!("Failed to get default keychain: {}", e)))?;

        keychain.unlock(password)
            .map_err(|e| LeashError::Backend(format!("Failed to unlock keychain: {}", e)))?;

                Ok(())

            }

        }

        

        #[cfg(test)]

        mod tests;

        