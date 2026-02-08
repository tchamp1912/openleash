use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tokio::process::Command;
use leash_ai_core::{Result, LeashError};
use leash_ai_core::config::NodeBootstrapConfig;

#[cfg(test)]
mod tests;

pub struct VenvManager {
    root_path: PathBuf,
}

impl VenvManager {
    pub fn new(root_path: impl Into<PathBuf>) -> Self {
        Self {
            root_path: root_path.into(),
        }
    }

    pub fn path_for_scope(&self) -> &Path {
        &self.root_path
    }

    pub fn bin_dir(&self) -> PathBuf {
        if cfg!(windows) {
            self.root_path.join("Scripts")
        } else {
            self.root_path.join("bin")
        }
    }

    pub fn brew_dir(&self) -> PathBuf {
        self.root_path.join(".brew")
    }

    pub fn brew_bin(&self) -> PathBuf {
        self.brew_dir().join("bin").join("brew")
    }

    /// Directory for the portable Node.js installation (mirrors "portable brew" isolation).
    pub fn node_dir(&self) -> PathBuf {
        self.root_path.join(".node")
    }

    /// Binary directory of the portable Node.js (node, npm, npx).
    pub fn node_bin_dir(&self) -> PathBuf {
        self.node_dir().join("bin")
    }

    /// Path to the portable `npm` binary. Use this instead of system npm for isolation.
    pub fn npm_bin(&self) -> PathBuf {
        self.node_bin_dir().join(if cfg!(windows) { "npm.cmd" } else { "npm" })
    }

    pub async fn ensure_created(&self) -> Result<()> {
        if !self.root_path.exists() {
            tracing::info!("Creating new venv at {:?}", self.root_path);
            let status = Command::new("python3")
                .arg("-m")
                .arg("venv")
                .arg(&self.root_path)
                .status()
                .await
                .map_err(|e| LeashError::Backend(format!("Failed to create venv: {}", e)))?;

            if !status.success() {
                return Err(LeashError::Backend(format!("python3 -m venv failed with status {}", status)));
            }
        }
        Ok(())
    }

    pub async fn ensure_brew_bootstrapped(&self) -> Result<()> {
        let brew_dir = self.brew_dir();
        if !brew_dir.exists() {
            tracing::info!("Bootstrapping local Homebrew in {:?}", brew_dir);
            tokio::fs::create_dir_all(&brew_dir).await
                .map_err(|e| LeashError::Backend(format!("Failed to create brew dir: {}", e)))?;

            let status = Command::new("git")
                .arg("clone")
                .arg("--depth=1")
                .arg("https://github.com/Homebrew/brew")
                .arg(&brew_dir)
                .status()
                .await
                .map_err(|e| LeashError::Backend(format!("Failed to clone Homebrew: {}", e)))?;

            if !status.success() {
                return Err(LeashError::Backend(format!("Homebrew clone failed with status {}", status)));
            }
        }
        Ok(())
    }

    /// Bootstrap a portable Node.js (and npm) into the scope so NPM runs are isolated from the system.
    /// Uses the configured version and dist URL (from `leash init` / config.yaml). Unix only in v0.
    pub async fn ensure_node_bootstrapped(&self, config: &NodeBootstrapConfig) -> Result<()> {
        let node_dir = self.node_dir();
        let npm_bin = self.npm_bin();
        if node_dir.exists() && npm_bin.exists() {
            return Ok(());
        }

        let version = config.version.trim();
        let base_url = config
            .dist_base_url
            .as_deref()
            .unwrap_or("https://nodejs.org/dist")
            .trim_end_matches('/');
        let (os, arch) = platform_for_node()?;
        let tarball_name = format!("node-v{}-{}-{}.tar.gz", version, os, arch);
        let url = format!("{}/v{}/{}", base_url, version, tarball_name);

        tracing::info!("Bootstrapping portable Node.js {} in {:?}", version, node_dir);
        tokio::fs::create_dir_all(&node_dir).await
            .map_err(|e| LeashError::Backend(format!("Failed to create node dir: {}", e)))?;

        let archive_path = node_dir.join(&tarball_name);

        let status = Command::new("curl")
            .arg("-sL")
            .arg("-o")
            .arg(&archive_path)
            .arg(&url)
            .status()
            .await
            .map_err(|e| LeashError::Backend(format!("Failed to run curl: {}", e)))?;
        if !status.success() {
            let _ = tokio::fs::remove_file(&archive_path).await;
            return Err(LeashError::Backend("Failed to download Node.js tarball".to_string()));
        }

        let status = Command::new("tar")
            .arg("-xzf")
            .arg(&archive_path)
            .arg("-C")
            .arg(&node_dir)
            .arg("--strip-components=1")
            .status()
            .await
            .map_err(|e| LeashError::Backend(format!("Failed to run tar: {}", e)))?;

        let _ = tokio::fs::remove_file(&archive_path).await;
        if !status.success() {
            return Err(LeashError::Backend("Failed to extract Node.js tarball".to_string()));
        }

        if !npm_bin.exists() {
            return Err(LeashError::Backend(format!(
                "npm binary not found at {:?} after extract",
                npm_bin
            )));
        }
        Ok(())
    }

    pub fn get_activation_env(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();
        let bin_path = self.bin_dir();
        let brew_bin_dir = self.brew_dir().join("bin");
        let node_bin_dir = self.node_bin_dir();

        // Python Venv activation
        env.insert("VIRTUAL_ENV".to_string(), self.root_path.to_string_lossy().to_string());

        // PATH: venv bin, then portable node, then portable brew, then system
        let current_path = std::env::var_os("PATH").unwrap_or_default();
        let mut paths = Vec::new();
        paths.push(bin_path.to_string_lossy().to_string());
        if node_bin_dir.exists() {
            paths.push(node_bin_dir.to_string_lossy().to_string());
        }
        if brew_bin_dir.exists() {
            paths.push(brew_bin_dir.to_string_lossy().to_string());
        }
        if !current_path.is_empty() {
            paths.push(current_path.to_string_lossy().to_string());
        }
        env.insert("PATH".to_string(), paths.join(":"));

        // Homebrew standalone mode
        if brew_bin_dir.exists() {
            env.insert("HOMEBREW_PREFIX".to_string(), self.brew_dir().to_string_lossy().to_string());
        }

        // NPM: global installs go into this scope (same as before; we now use portable npm)
        env.insert("NPM_CONFIG_PREFIX".to_string(), self.root_path.to_string_lossy().to_string());

        env
    }

    pub async fn remove(&self) -> Result<()> {
        if self.root_path.exists() {
            tokio::fs::remove_dir_all(&self.root_path)
                .await
                .map_err(|e| LeashError::Backend(format!("Failed to remove venv: {}", e)))?;
        }
        Ok(())
    }
}

/// Map Rust OS/arch to Node.js official tarball naming (nodejs.org/dist).
fn platform_for_node() -> Result<(String, String)> {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        "linux" => "linux",
        "windows" => {
            return Err(LeashError::Backend(
                "Portable Node bootstrap is not yet supported on Windows; use system npm or add win support".to_string(),
            ));
        }
        other => {
            return Err(LeashError::Backend(format!(
                "Unsupported OS for portable Node: {}",
                other
            )));
        }
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" | "arm64" => "arm64",
        "arm" => "armv7l",
        other => {
            return Err(LeashError::Backend(format!(
                "Unsupported arch for portable Node: {}",
                other
            )));
        }
    };
    Ok((os.to_string(), arch.to_string()))
}
