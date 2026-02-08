use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenLeashConfig {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub backends: BackendConfig,
    /// Drives portable Node.js (npm) bootstrap per scope. Set at init; used by the NPM backend.
    #[serde(default)]
    pub node_bootstrap: NodeBootstrapConfig,
    pub telegram: Option<TelegramConfig>,
}

/// Config for bootstrapping a portable Node.js (and npm) into each task scope.
/// Written by `leash init`; daemon uses it when the NPM backend runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeBootstrapConfig {
    /// Node.js version (e.g. "20.18.0" LTS). Must match a tarball at nodejs.org/dist.
    pub version: String,
    /// Base URL for Node dist (default: https://nodejs.org/dist). Use for mirrors or air-gap.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dist_base_url: Option<String>,
}

impl Default for NodeBootstrapConfig {
    fn default() -> Self {
        Self {
            version: "20.18.0".to_string(),
            dist_base_url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub token: String,
    pub chat_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub uds_path: String,
    pub tcp_host: String,
    pub tcp_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub database_url: String,
    pub policies_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub pip: FeatureConfig,
    pub npm: FeatureConfig,
    pub brew: FeatureConfig,
    pub keychain: FeatureConfig,
    pub telegram: FeatureConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub enabled: bool,
}

impl Default for OpenLeashConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let config_dir = format!("{}/.openleash", home);
        
        Self {
            server: ServerConfig {
                uds_path: "/tmp/openleash.sock".to_string(),
                tcp_host: "127.0.0.1".to_string(),
                tcp_port: 50051,
            },
            storage: StorageConfig {
                database_url: format!("sqlite://{}/leash.db", config_dir),
                policies_path: Some(format!("{}/policies.yaml", config_dir)),
            },
            backends: BackendConfig {
                pip: FeatureConfig { enabled: true },
                npm: FeatureConfig { enabled: true },
                brew: FeatureConfig { enabled: true },
                keychain: FeatureConfig { enabled: true },
                telegram: FeatureConfig { enabled: false },
            },
            node_bootstrap: NodeBootstrapConfig::default(),
            telegram: None,
        }
    }
}

impl OpenLeashConfig {
    /// Node bootstrap config to use for the NPM backend.
    pub fn node_bootstrap_config(&self) -> NodeBootstrapConfig {
        self.node_bootstrap.clone()
    }

    pub fn load(path: Option<PathBuf>) -> Result<Self, crate::error::OpenLeashError> {
        let path = path.unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(format!("{}/.openleash/config.yaml", home))
        });

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::error::OpenLeashError::Config(format!("Failed to read config file: {}", e)))?;
        
        serde_yaml::from_str(&content)
            .map_err(|e| crate::error::OpenLeashError::Config(format!("Failed to parse config YAML: {}", e)))
    }
}
