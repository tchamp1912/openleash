use std::collections::HashSet;
use crate::config::OpenLeashConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxLevel {
    Permissive,
    Restrictive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkMode {
    Open,
    Closed,
}

pub struct SandboxProfileGenerator {
    level: SandboxLevel,
    network: NetworkMode,
    features: HashSet<String>,
    home_dir: String,
    project_dir: String,
}

impl SandboxProfileGenerator {
    pub fn new(level: SandboxLevel, network: NetworkMode, home_dir: &str, project_dir: &str) -> Self {
        Self {
            level,
            network,
            features: HashSet::new(),
            home_dir: home_dir.to_string(),
            project_dir: project_dir.to_string(),
        }
    }

    pub fn with_config(mut self, config: &OpenLeashConfig) -> Self {
        if config.backends.pip.enabled { self.features.insert("pip".to_string()); }
        if config.backends.npm.enabled { self.features.insert("npm".to_string()); }
        if config.backends.brew.enabled { self.features.insert("brew".to_string()); }
        if config.backends.keychain.enabled { self.features.insert("keychain".to_string()); }
        self
    }

    pub fn generate(&self) -> String {
        let mut sb = String::from(";; OpenLeash Agent Sandbox Profile
(version 1)
(deny default)

");

        // --- Core System Basics ---
        sb.push_str(";; Basic system libraries and metadata
");
        sb.push_str(r#"(allow file-read* (subpath "/usr/lib") (subpath "/usr/share") (subpath "/System/Library"))
"#);
        sb.push_str(r#"(allow file-read-metadata (subpath "/"))
"#);

        // --- Level Specifics ---
        match self.level {
            SandboxLevel::Permissive => {
                sb.push_str(";; Permissive: Allow reading most of the filesystem
");
                sb.push_str(r#"(allow file-read* (subpath "/"))
"#);
                // Deny sensitive areas even in permissive
                sb.push_str(r#"(deny file-read* (subpath "/Users/shared") (subpath "/var/db"))
"#);
            }
            SandboxLevel::Restrictive => {
                sb.push_str(";; Restrictive: Only allow reading project and Leash config
");
                sb.push_str(&format!(r#"(allow file-read* (subpath "{}"))
"#, self.project_dir));
                sb.push_str(&format!(r#"(allow file-read* (subpath "{}/.openleash"))
"#, self.home_dir));
            }
        }

        // --- Network ---
        match self.network {
            NetworkMode::Open => {
                sb.push_str("
;; Network: Allowed
(allow network-outbound)
(allow network-inbound)
");
            }
            NetworkMode::Closed => {
                sb.push_str("
;; Network: Blocked
(deny network-outbound)
(deny network-inbound)
");
                // Allow loopback for local IPC if needed
                sb.push_str(r#"(allow network-outbound (literal "localhost") (literal "127.0.0.1"))
"#);
            }
        }

        // --- Leash Infrastructure ---
        sb.push_str("
;; Leash Infrastructure
");
        sb.push_str(r#"(allow file-read* file-write* (literal "/tmp/openleash.sock"))
"#);
        sb.push_str(r#"(allow file-read* file-write* (subpath "/tmp/openleash-tasks"))
"#);
        sb.push_str(r#"(allow process-exec (subpath "/tmp/openleash-tasks"))
"#);
        sb.push_str(r#"(allow file-write* (subpath "/dev/stdout") (subpath "/dev/stderr") (subpath "/dev/tty"))
"#);

        // --- Features ---
        if self.features.contains("pip") {
            sb.push_str("
;; Feature: Python/Pip
");
            sb.push_str(r#"(allow process-exec (literal "/usr/bin/python3"))
"#);
            // Add other common python locations
            sb.push_str(r#"(allow file-read* (subpath "/Library/Developer/CommandLineTools/usr/bin"))
"#);
        }

        if self.features.contains("brew") {
            sb.push_str("
;; Feature: Homebrew
");
            sb.push_str(r#"(allow file-read* (subpath "/opt/homebrew"))
"#);
            sb.push_str(r#"(allow process-exec (subpath "/opt/homebrew"))
"#);
        }

        if self.features.contains("keychain") {
            sb.push_str("
;; Feature: macOS Keychain (IPC to securityd)
");
            sb.push_str(r#"(allow mach-lookup (global-name "com.apple.securityd"))
"#);
            sb.push_str(r#"(allow mach-lookup (global-name "com.apple.SecurityServer"))
"#);
        }

        sb
    }
}