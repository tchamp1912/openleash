use crate::VenvManager;
use tempfile::tempdir;

#[tokio::test]
async fn test_venv_path_resolution() {
    let dir = tempdir().unwrap();
    let manager = VenvManager::new(dir.path());
    
    let bin_dir = manager.bin_dir();
    if cfg!(windows) {
        assert_eq!(bin_dir, dir.path().join("Scripts"));
    } else {
        assert_eq!(bin_dir, dir.path().join("bin"));
    }
}

#[tokio::test]
async fn test_venv_activation_env() {
    let dir = tempdir().unwrap();
    let manager = VenvManager::new(dir.path());
    
    let env = manager.get_activation_env();
    
    assert_eq!(env.get("VIRTUAL_ENV").unwrap(), &dir.path().to_string_lossy().to_string());
    assert!(env.get("PATH").unwrap().contains(&dir.path().join("bin").to_string_lossy().to_string()));
    assert_eq!(env.get("NPM_CONFIG_PREFIX").unwrap(), &dir.path().to_string_lossy().to_string());
}
