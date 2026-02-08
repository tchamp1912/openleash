use async_trait::async_trait;
use leash_ai_backend::PackageBackend;
use leash_ai_core::{Result, LeashError};
use leash_ai_venv::VenvManager;
use tokio::process::Command;
use std::path::PathBuf;

pub struct BrewBackend;

#[async_trait]
impl PackageBackend for BrewBackend {
    async fn install(&self, package: &str, version: Option<&str>, scope_path: &str) -> Result<()> {
        let venv = VenvManager::new(scope_path);
        venv.ensure_created().await?;
        venv.ensure_brew_bootstrapped().await?;

        let pkg_spec = match version {
            Some(v) => format!("{}@{}", package, v),
            None => package.to_string(),
        };

        // Use the venv's activation env to ensure brew knows its prefix
        let envs = venv.get_activation_env();

        let status = Command::new(venv.brew_bin())
            .arg("install")
            .arg(pkg_spec)
            .envs(&envs)
            .status()
            .await
            .map_err(|e| LeashError::Backend(format!("Failed to run local brew: {}", e)))?;

        if !status.success() {
            return Err(LeashError::Backend(format!("Local brew install exited with status: {}", status)));
        }

        Ok(())
    }

    async fn uninstall(&self, scope_path: &str) -> Result<()> {
        let venv = VenvManager::new(scope_path);
        venv.remove().await
    }

    fn executable_directory(&self, scope_path: &str) -> PathBuf {
        VenvManager::new(scope_path).brew_dir().join("bin")
    }
}

#[cfg(test)]
mod tests;