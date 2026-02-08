use crate::LeashDaemon;
use leash_ai_api::pb::request_service_server::RequestService;
use leash_ai_api::pb::task_service_server::TaskService;
use leash_ai_api::pb::{ExecuteCommandRequest, StartTaskRequest};
use leash_ai_core::models::{Policy, ResourceType, ApprovalScope};
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
    let policies = vec![
        Policy {
            id: "allow-all".to_string(),
            name: "Allow All".to_string(),
            description: None,
            resource_type: ResourceType::Command,
            priority: 0,
            allowed_patterns: vec![".*".to_string()],
            auto_approve: true,
            max_ttl_seconds: 0,
            default_scope: ApprovalScope::Once,
        }
    ];
    let daemon = Arc::new(LeashDaemon {
        db,
        pip_backend: Arc::new(PipBackend),
        brew_backend: Arc::new(BrewBackend),
        npm_backend: Arc::new(NpmBackend::default()),
        keychain_backend: Arc::new(KeychainBackend),
        policy_engine: Arc::new(PolicyEngine::new(policies)),
        approval_backends: vec![],
    });
    
    let start_res = daemon.start_task(Request::new(StartTaskRequest {
        name: "test".to_string(),
        ttl_seconds: 3600,
        base_scope_path: "/tmp/leash-lifecycle-tests".to_string(),
    })).await.unwrap().into_inner();

    let bin_dir = std::path::PathBuf::from(&start_res.scope_path).join("bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let script_path = bin_dir.join("my-tool");
    std::fs::write(&script_path, "#!/bin/sh\necho 'expanded'").unwrap();
    
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(&script_path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&script_path, perms).unwrap();

    let exec_res = daemon.execute_command(Request::new(ExecuteCommandRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        command: "my-tool".to_string(),
        args: vec![],
        reason: "test".to_string(),
        task_id: Some(start_res.task_id),
        env_vars: std::collections::HashMap::new(),
        working_dir: None,
        timeout_seconds: 5,
    })).await.unwrap().into_inner();

    assert_eq!(exec_res.stdout.trim(), "expanded");
    let _ = std::fs::remove_dir_all(&start_res.scope_path);
}