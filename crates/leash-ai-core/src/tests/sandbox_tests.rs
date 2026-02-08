use crate::sandbox::{SandboxProfileGenerator, SandboxLevel, NetworkMode};
use crate::config::LeashConfig;

#[test]
fn test_sandbox_profile_generation() {
    let home = "/Users/test";
    let project = "/Users/test/projects/leash";
    let gen = SandboxProfileGenerator::new(SandboxLevel::Restrictive, NetworkMode::Closed, home, project);
    
    let profile = gen.generate();
    
    // Check for restrictive file access
    assert!(profile.contains(&format!("(allow file-read* (subpath \"{}\"))", project)));
    assert!(profile.contains(&format!("(allow file-read* (subpath \"{}/.leash\"))", home)));
    
    // Check for closed network
    assert!(profile.contains("(deny network-outbound)"));
    assert!(profile.contains("(allow network-outbound (literal \"localhost\") (literal \"127.0.0.1\"))"));
}

#[test]
fn test_sandbox_feature_activation() {
    let mut config = LeashConfig::default();
    config.backends.brew.enabled = true;
    
    let gen = SandboxProfileGenerator::new(SandboxLevel::Permissive, NetworkMode::Open, "/home", "/proj")
        .with_config(&config);
    
    let profile = gen.generate();
    
    // Check for brew specific holes
    assert!(profile.contains(";; Feature: Homebrew"));
    assert!(profile.contains("(allow file-read* (subpath \"/opt/homebrew\"))"));
}