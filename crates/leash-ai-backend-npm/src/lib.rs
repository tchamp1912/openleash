use async_trait::async_trait;
use leash_ai_backend::PackageBackend;
use leash_ai_core::config::NodeBootstrapConfig;
use leash_ai_core::{Result, LeashError};
use leash_ai_venv::VenvManager;
use std::path::PathBuf;
use tokio::process::Command;

pub struct NpmBackend {
    pub(crate) node_config: NodeBootstrapConfig,
}

impl NpmBackend {
    pub fn new(node_config: NodeBootstrapConfig) -> Self {
        Self { node_config }
    }
}

impl Default for NpmBackend {
    fn default() -> Self {
        Self::new(NodeBootstrapConfig::default())
    }
}

#[async_trait]
impl PackageBackend for NpmBackend {
    async fn install(&self, package: &str, version: Option<&str>, scope_path: &str) -> Result<()> {
        let venv = VenvManager::new(scope_path);
        venv.ensure_created().await?;
        venv.ensure_node_bootstrapped(&self.node_config).await?;

        let pkg_spec = match version {
            Some(v) => format!("{}@{}", package, v),
            None => package.to_string(),
        };

        let envs = venv.get_activation_env();
        let npm_bin = venv.npm_bin();

        let status = Command::new(&npm_bin)
            .arg("install")
            .arg("-g") // NPM_CONFIG_PREFIX points at scope, so -g installs into scope
            .arg(pkg_spec)
            .envs(&envs)
            .status()
            .await
            .map_err(|e| LeashError::Backend(format!("Failed to run npm: {}", e)))?;

        if !status.success() {
            return Err(LeashError::Backend(format!("npm install exited with status: {}", status)));
        }

        Ok(())
    }

    async fn uninstall(&self, scope_path: &str) -> Result<()> {
        let venv = VenvManager::new(scope_path);
        venv.remove().await
    }

    fn executable_directory(&self, scope_path: &str) -> PathBuf {
        // Global npm installs go into scope_path/bin (NPM_CONFIG_PREFIX=scope_path)
        VenvManager::new(scope_path).bin_dir()
    }
}

#[cfg(test)]
mod tests;
