use crate::LeashDaemon;
use leash_ai_api::pb::task_service_server::TaskService;
use leash_ai_api::pb::{GetTaskEnvironmentRequest, StartTaskRequest};
use leash_ai_core::policy::PolicyEngine;
use leash_ai_db::db::Db;
use leash_ai_backend_pip::PipBackend;
use leash_ai_backend_brew::BrewBackend;
use leash_ai_backend_npm::NpmBackend;
use leash_ai_backend_keychain::KeychainBackend;
use std::sync::Arc;
use tonic::Request;

#[tokio::test]
async fn test_task_path_expansion() {
    let db = Arc::new(Db::new("sqlite::memory:").await.unwrap());
    let daemon = Arc::new(LeashDaemon {
        db,
        pip_backend: Arc::new(PipBackend),
        brew_backend: Arc::new(BrewBackend),
        npm_backend: Arc::new(NpmBackend::default()),
        keychain_backend: Arc::new(KeychainBackend),
        policy_engine: Arc::new(PolicyEngine::new(vec![])),
        approval_backends: vec![],
    });
    
    let start_res = daemon.start_task(Request::new(StartTaskRequest {
        name: "test".to_string(),
        ttl_seconds: 3600,
        base_scope_path: "/tmp/leash-lifecycle-tests".to_string(),
    })).await.unwrap().into_inner();

    // Get task environment and verify bin_path is correct
    let env_res = daemon.get_task_environment(Request::new(GetTaskEnvironmentRequest {
        task_id: start_res.task_id.clone(),
    })).await.unwrap().into_inner();

    assert_eq!(env_res.scope_path, start_res.scope_path);
    assert_eq!(env_res.bin_path, format!("{}/bin", start_res.scope_path));
    assert!(env_res.error_message.is_empty());
    
    let _ = std::fs::remove_dir_all(&start_res.scope_path);
}