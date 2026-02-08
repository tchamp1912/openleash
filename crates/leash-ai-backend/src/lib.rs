use async_trait::async_trait;
use leash_ai_core::Result;
use std::path::PathBuf;

#[async_trait]
pub trait PackageBackend: Send + Sync {
    async fn install(&self, package: &str, version: Option<&str>, scope_path: &str) -> Result<()>;
    async fn uninstall(&self, scope_path: &str) -> Result<()>;

    /// Directory containing installed executables (e.g. venv `bin/` or `Scripts/`).
    /// Callers use this to set `PATH` or to resolve the pip/python binary.
    fn executable_directory(&self, scope_path: &str) -> PathBuf;
}

#[async_trait]
pub trait SecretBackend: Send + Sync {
    /// Retrieve a secret value by its identifier (e.g., "anthropic/api-key").
    async fn get_secret(&self, secret_id: &str) -> Result<String>;

    /// Store a secret value.
    async fn store_secret(&self, secret_id: &str, value: &str) -> Result<()>;

    /// Check if the secret store is currently locked.
    async fn is_locked(&self) -> Result<bool>;

    /// Attempt to unlock the secret store with an optional password.
    /// In production, this should be handled carefully to avoid leaking passwords.
    async fn unlock(&self, password: Option<&str>) -> Result<()>;
}

#[async_trait]
pub trait ApprovalBackend: Send + Sync {
    /// Notify approvers about a new request.
    async fn notify_approval(&self, request: &leash_ai_core::models::ApprovalRequest) -> Result<()>;
}
