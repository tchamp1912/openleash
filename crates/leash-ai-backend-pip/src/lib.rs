use async_trait::async_trait;
use leash_ai_backend::PackageBackend;
use leash_ai_core::{Result, LeashError};
use leash_ai_venv::VenvManager;
use tokio::process::Command;
use std::path::PathBuf;

pub struct PipBackend;

#[async_trait]
impl PackageBackend for PipBackend {
    async fn install(&self, package: &str, version: Option<&str>, scope_path: &str) -> Result<()> {
        let venv = VenvManager::new(scope_path);
        venv.ensure_created().await?;

        let pip_exe = venv.bin_dir().join("pip");
        let pkg_spec = match version {
            Some(v) => format!("{}=={}", package, v),
            None => package.to_string(),
        };

        let status = Command::new(pip_exe)
            .arg("install")
            .arg(pkg_spec)
            .status()
            .await
            .map_err(|e| LeashError::Backend(format!("Failed to run pip: {}", e)))?;

        if !status.success() {
            return Err(LeashError::Backend(format!("pip install exited with status: {}", status)));
        }

        Ok(())
    }

    async fn uninstall(&self, scope_path: &str) -> Result<()> {
        let venv = VenvManager::new(scope_path);
        venv.remove().await
    }

        fn executable_directory(&self, scope_path: &str) -> PathBuf {

            VenvManager::new(scope_path).bin_dir()

        }

    }

    

    #[cfg(test)]

    mod tests;

    