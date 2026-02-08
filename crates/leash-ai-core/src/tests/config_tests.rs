use crate::config::LeashConfig;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_default_config() {
    let config = LeashConfig::default();
    assert_eq!(config.server.tcp_port, 50051);
    assert!(config.backends.pip.enabled);
    assert!(config.backends.npm.enabled);
    assert_eq!(config.node_bootstrap.version, "20.18.0");
    assert!(config.node_bootstrap.dist_base_url.is_none());
}

#[test]
fn test_load_config_from_yaml() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.yaml");
    
    let yaml = r#"
server:
  uds_path: "/tmp/custom.sock"
  tcp_host: "0.0.0.0"
  tcp_port: 9000
storage:
  database_url: "sqlite://test.db"
  policies_path: "/etc/leash/policies.yaml"
backends:
  pip:
    enabled: true
  npm:
    enabled: false
  brew:
    enabled: true
  keychain:
    enabled: true
  telegram:
    enabled: false
"#;
    fs::write(&config_path, yaml).unwrap();

    let config = LeashConfig::load(Some(config_path)).unwrap();
    assert_eq!(config.server.tcp_port, 9000);
    assert_eq!(config.server.uds_path, "/tmp/custom.sock");
    assert!(!config.backends.npm.enabled);
    // node_bootstrap omitted in YAML gets default
    assert_eq!(config.node_bootstrap.version, "20.18.0");
}

#[test]
fn test_load_config_with_node_bootstrap() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.yaml");
    let yaml = r#"
server:
  uds_path: "/tmp/leash.sock"
  tcp_host: "127.0.0.1"
  tcp_port: 50051
storage:
  database_url: "sqlite://leash.db"
backends:
  pip: { enabled: true }
  npm: { enabled: true }
  brew: { enabled: true }
  keychain: { enabled: true }
  telegram: { enabled: false }
node_bootstrap:
  version: "22.11.0"
  dist_base_url: "https://internal-mirror.example.com/node"
"#;
    fs::write(&config_path, yaml).unwrap();
    let config = LeashConfig::load(Some(config_path)).unwrap();
    assert_eq!(config.node_bootstrap.version, "22.11.0");
    assert_eq!(
        config.node_bootstrap.dist_base_url.as_deref(),
        Some("https://internal-mirror.example.com/node")
    );
}
