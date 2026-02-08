use crate::LeashDaemon;
use leash_ai_api::pb::request_service_server::RequestService;
use leash_ai_api::pb::approval_service_server::ApprovalService;
use leash_ai_api::pb::task_service_server::TaskService;
use leash_ai_api::pb::{RequestSecretRequest, ListPendingApprovalsRequest, ApproveRequest, StartTaskRequest, ExecuteCommandRequest};
use leash_ai_core::models::{Policy, ResourceType, ApprovalScope};
use leash_ai_core::policy::PolicyEngine;
use leash_ai_db::db::Db;
use leash_ai_backend_pip::PipBackend;
use leash_ai_backend_brew::BrewBackend;
use leash_ai_backend_npm::NpmBackend;
use leash_ai_backend_keychain::KeychainBackend;
use std::sync::Arc;
use tonic::Request;
use uuid::Uuid;
use std::collections::HashMap;

async fn setup_daemon_with_policies(policies: Vec<Policy>) -> Arc<LeashDaemon> {
    let db = Arc::new(Db::new("sqlite::memory:").await.unwrap());
    let policy_engine = Arc::new(PolicyEngine::new(policies));
    
    Arc::new(LeashDaemon {
        db,
        pip_backend: Arc::new(PipBackend),
        brew_backend: Arc::new(BrewBackend),
        npm_backend: Arc::new(NpmBackend::default()),
        keychain_backend: Arc::new(KeychainBackend),
        policy_engine,
        approval_backends: vec![],
    })
}

#[tokio::test]
async fn test_permanent_approval_flow() {
    let policies = vec![
        Policy {
            id: "secret-policy".to_string(),
            name: "Secret Policy".to_string(),
            description: None,
            resource_type: ResourceType::Secret,
            priority: 10,
            allowed_patterns: vec![".*".to_string()],
            max_ttl_seconds: 3600,
            auto_approve: false,
            default_scope: ApprovalScope::Permanent,
        }
    ];
    let daemon = setup_daemon_with_policies(policies).await;

    let request_id = Uuid::new_v4().to_string();
    let secret_id = "test-secret".to_string();

    // 1. Initial request - should be pending
    let req = Request::new(RequestSecretRequest {
        request_id: request_id.clone(),
        secret_id: secret_id.clone(),
        reason: "testing".to_string(),
        task_id: None,
    });
    
    let res = daemon.request_secret(req).await.unwrap().into_inner();
    assert_eq!(res.status, "PENDING_APPROVAL");

    // 2. Approve permanently
    let list_res = daemon.list_pending_approvals(Request::new(ListPendingApprovalsRequest {})).await.unwrap().into_inner();
    assert_eq!(list_res.approvals.len(), 1);
    let approval_id = list_res.approvals[0].approval_id.clone();

    daemon.approve(Request::new(ApproveRequest {
        approval_id,
        scope: Some("permanent".to_string()),
    })).await.unwrap();

    // 3. Second request - should be pre-approved (or at least not PENDING_APPROVAL)
    // Note: It might be DENIED if the keychain doesn't have the secret, but it shouldn't be PENDING_APPROVAL
    let req2 = Request::new(RequestSecretRequest {
        request_id: Uuid::new_v4().to_string(),
        secret_id: secret_id.clone(),
        reason: "testing again".to_string(),
        task_id: None,
    });
    
    let res2 = daemon.request_secret(req2).await.unwrap().into_inner();
    assert_ne!(res2.status, "PENDING_APPROVAL");
}

#[tokio::test]
async fn test_task_scoped_approval_flow() {
    let policies = vec![
        Policy {
            id: "cmd-policy".to_string(),
            name: "Command Policy".to_string(),
            description: None,
            resource_type: ResourceType::Command,
            priority: 10,
            allowed_patterns: vec!["ls".to_string()],
            max_ttl_seconds: 3600,
            auto_approve: false,
            default_scope: ApprovalScope::Task,
        }
    ];
    let daemon = setup_daemon_with_policies(policies).await;

    // 1. Start a task
    let task_res = daemon.start_task(Request::new(StartTaskRequest {
        name: "test-task".to_string(),
        ttl_seconds: 3600,
        base_scope_path: "/tmp".to_string(),
    })).await.unwrap().into_inner();
    let task_id = task_res.task_id;

    // 2. Request command within task - should be pending
    let req = Request::new(ExecuteCommandRequest {
        request_id: Uuid::new_v4().to_string(),
        command: "ls".to_string(),
        args: vec![],
        reason: "testing".to_string(),
        task_id: Some(task_id.clone()),
        env_vars: HashMap::new(),
        working_dir: None,
        timeout_seconds: 10,
    });
    
    let res = daemon.execute_command(req).await.unwrap().into_inner();
    assert_eq!(res.status, "PENDING_APPROVAL");

    // 3. Approve for task
    let list_res = daemon.list_pending_approvals(Request::new(ListPendingApprovalsRequest {})).await.unwrap().into_inner();
    let approval_id = list_res.approvals[0].approval_id.clone();

    daemon.approve(Request::new(ApproveRequest {
        approval_id,
        scope: Some("task".to_string()),
    })).await.unwrap();

    // 4. Second request within SAME task - should be pre-approved
    let req2 = Request::new(ExecuteCommandRequest {
        request_id: Uuid::new_v4().to_string(),
        command: "ls".to_string(),
        args: vec![],
        reason: "testing again".to_string(),
        task_id: Some(task_id.clone()),
        env_vars: HashMap::new(),
        working_dir: None,
        timeout_seconds: 10,
    });
    
    let res2 = daemon.execute_command(req2).await.unwrap().into_inner();
    assert_ne!(res2.status, "PENDING_APPROVAL");

    // 5. Start another task
    let task_res2 = daemon.start_task(Request::new(StartTaskRequest {
        name: "test-task-2".to_string(),
        ttl_seconds: 3600,
        base_scope_path: "/tmp".to_string(),
    })).await.unwrap().into_inner();
    let task_id2 = task_res2.task_id;

    // 6. Request command within DIFFERENT task - should be pending again
    let req3 = Request::new(ExecuteCommandRequest {
        request_id: Uuid::new_v4().to_string(),
        command: "ls".to_string(),
        args: vec![],
        reason: "testing in new task".to_string(),
        task_id: Some(task_id2),
        env_vars: HashMap::new(),
        working_dir: None,
        timeout_seconds: 10,
    });
    
    let res3 = daemon.execute_command(req3).await.unwrap().into_inner();
    assert_eq!(res3.status, "PENDING_APPROVAL");
}

#[tokio::test]
async fn test_denied_approval_flow() {
    let policies = vec![
        Policy {
            id: "secret-policy".to_string(),
            name: "Secret Policy".to_string(),
            description: None,
            resource_type: ResourceType::Secret,
            priority: 10,
            allowed_patterns: vec![".*".to_string()],
            max_ttl_seconds: 3600,
            auto_approve: false,
            default_scope: ApprovalScope::Once,
        }
    ];
    let daemon = setup_daemon_with_policies(policies).await;

    let req = Request::new(RequestSecretRequest {
        request_id: Uuid::new_v4().to_string(),
        secret_id: "test-secret".to_string(),
        reason: "testing".to_string(),
        task_id: None,
    });
    
    let res = daemon.request_secret(req).await.unwrap().into_inner();
    assert_eq!(res.status, "PENDING_APPROVAL");

    // Deny
    let list_res = daemon.list_pending_approvals(Request::new(ListPendingApprovalsRequest {})).await.unwrap().into_inner();
    let approval_id = list_res.approvals[0].approval_id.clone();

    daemon.deny(Request::new(leash_ai_api::pb::DenyRequest {
        approval_id,
    })).await.unwrap();

    // Second request with SAME request_id - should be denied
    let req2 = Request::new(RequestSecretRequest {
        request_id: res.request_id.clone(),
        secret_id: "test-secret".to_string(),
        reason: "testing again".to_string(),
        task_id: None,
    });
    
    let res2 = daemon.request_secret(req2).await;
    assert!(res2.is_err());
    assert_eq!(res2.unwrap_err().code(), tonic::Code::PermissionDenied);
}

#[tokio::test]
async fn test_once_approval_flow() {
    let policies = vec![
        Policy {
            id: "pkg-policy".to_string(),
            name: "Pkg Policy".to_string(),
            description: None,
            resource_type: ResourceType::Package,
            priority: 10,
            allowed_patterns: vec![".*".to_string()],
            max_ttl_seconds: 3600,
            auto_approve: false,
            default_scope: ApprovalScope::Once,
        }
    ];
    let daemon = setup_daemon_with_policies(policies).await;

    // 1. First request
    let req = Request::new(leash_ai_api::pb::RequestPackageRequest {
        request_id: Uuid::new_v4().to_string(),
        manager: "pip".to_string(),
        package: "requests".to_string(),
        reason: "testing".to_string(),
        ttl_seconds: 3600,
        scope_path: "/tmp".to_string(),
        task_id: None,
    });
    
    let res = daemon.request_package(req).await.unwrap().into_inner();
    assert_eq!(res.status, "PENDING_APPROVAL");

    // Approve
    let list_res = daemon.list_pending_approvals(Request::new(ListPendingApprovalsRequest {})).await.unwrap().into_inner();
    let approval_id = list_res.approvals[0].approval_id.clone();

    daemon.approve(Request::new(ApproveRequest {
        approval_id,
        scope: Some("once".to_string()),
    })).await.unwrap();

    // 2. Second request with DIFFERENT request_id - should be pending again because scope was "once"
    let req2 = Request::new(leash_ai_api::pb::RequestPackageRequest {
        request_id: Uuid::new_v4().to_string(),
        manager: "pip".to_string(),
        package: "requests".to_string(),
        reason: "testing again".to_string(),
        ttl_seconds: 3600,
        scope_path: "/tmp".to_string(),
        task_id: None,
    });
    
    let res2 = daemon.request_package(req2).await.unwrap().into_inner();
    assert_eq!(res2.status, "PENDING_APPROVAL");
}

#[tokio::test]
async fn test_store_secret_approval_flow() {
    let policies = vec![
        Policy {
            id: "secret-store-policy".to_string(),
            name: "Secret Store Policy".to_string(),
            description: None,
            resource_type: ResourceType::Secret,
            priority: 10,
            allowed_patterns: vec!["db/.*".to_string()],
            max_ttl_seconds: 3600,
            auto_approve: false,
            default_scope: ApprovalScope::Permanent,
        }
    ];
    let daemon = setup_daemon_with_policies(policies).await;

    let req = Request::new(leash_ai_api::pb::StoreSecretRequest {
        request_id: Uuid::new_v4().to_string(),
        secret_id: "db/password".to_string(),
        value: "supersecret".to_string(),
        reason: "setting up db".to_string(),
        task_id: None,
    });
    
    // 1. Initial store request - should return success: false (because pending)
    let res = daemon.store_secret(req).await.unwrap().into_inner();
    assert!(!res.success);
    assert!(res.error_message.contains("pending human approval"));

    // 2. Approve
    let list_res = daemon.list_pending_approvals(Request::new(ListPendingApprovalsRequest {})).await.unwrap().into_inner();
    let approval_id = list_res.approvals[0].approval_id.clone();

    daemon.approve(Request::new(ApproveRequest {
        approval_id,
        scope: Some("permanent".to_string()),
    })).await.unwrap();

    // 3. Second store request - should succeed (it will call backend, which might fail on non-macOS but should at least NOT be pending)
    let req2 = Request::new(leash_ai_api::pb::StoreSecretRequest {
        request_id: Uuid::new_v4().to_string(),
        secret_id: "db/password".to_string(),
        value: "supersecret".to_string(),
        reason: "setting up db again".to_string(),
        task_id: None,
    });
    
    let res2 = daemon.store_secret(req2).await.unwrap().into_inner();
    // On macOS this might be true. On other platforms it might be false but with a different error message.
    if !res2.success {
        assert!(!res2.error_message.contains("pending human approval"));
    }
}

#[tokio::test]
async fn test_task_cleanup_revokes_approvals() {
    let policies = vec![
        Policy {
            id: "cmd-policy".to_string(),
            name: "Command Policy".to_string(),
            description: None,
            resource_type: ResourceType::Command,
            priority: 10,
            allowed_patterns: vec![".*".to_string()],
            max_ttl_seconds: 3600,
            auto_approve: false,
            default_scope: ApprovalScope::Task,
        }
    ];
    let daemon = setup_daemon_with_policies(policies).await;

    // 1. Start task
    let task_res = daemon.start_task(Request::new(StartTaskRequest {
        name: "cleanup-test".to_string(),
        ttl_seconds: 3600,
        base_scope_path: "/tmp".to_string(),
    })).await.unwrap().into_inner();
    let task_id = task_res.task_id;

    // 2. Request and Approve
    let req = Request::new(ExecuteCommandRequest {
        request_id: Uuid::new_v4().to_string(),
        command: "ls".to_string(),
        args: vec![],
        reason: "testing".to_string(),
        task_id: Some(task_id.clone()),
        env_vars: HashMap::new(),
        working_dir: None,
        timeout_seconds: 10,
    });
    daemon.execute_command(req).await.unwrap();

    let list_res = daemon.list_pending_approvals(Request::new(ListPendingApprovalsRequest {})).await.unwrap().into_inner();
    daemon.approve(Request::new(ApproveRequest {
        approval_id: list_res.approvals[0].approval_id.clone(),
        scope: Some("task".to_string()),
    })).await.unwrap();

    // Verify it's approved
    let req2 = Request::new(ExecuteCommandRequest {
        request_id: Uuid::new_v4().to_string(),
        command: "ls".to_string(),
        args: vec![],
        reason: "testing again".to_string(),
        task_id: Some(task_id.clone()),
        env_vars: HashMap::new(),
        working_dir: None,
        timeout_seconds: 10,
    });
    let res2 = daemon.execute_command(req2).await.unwrap().into_inner();
    assert_ne!(res2.status, "PENDING_APPROVAL");

    // 3. End task (Cleanup)
    daemon.end_task(Request::new(leash_ai_api::pb::EndTaskRequest {
        task_id: task_id.clone(),
    })).await.unwrap();

    // 4. Verification: check_approval for that task_id should now return false
    // Since we don't have a direct check_approval RPC, we can try to request it again with SAME task_id (if daemon allowed it)
    // Actually, daemon might deny it because task is no longer active.
    
    let req3 = Request::new(ExecuteCommandRequest {
        request_id: Uuid::new_v4().to_string(),
        command: "ls".to_string(),
        args: vec![],
        reason: "testing after cleanup".to_string(),
        task_id: Some(task_id.clone()),
        env_vars: HashMap::new(),
        working_dir: None,
        timeout_seconds: 10,
    });
    
    let res3 = daemon.execute_command(req3).await;
    // It should be an error because Task is not Active
    assert!(res3.is_err());
    assert!(res3.unwrap_err().message().contains("Task is no longer active"));

    // Check DB directly if we want to be sure approved_resources is empty
    let db_approved = daemon.db.check_approval(ResourceType::Command, "ls", Some(Uuid::parse_str(&task_id).unwrap())).await.unwrap();
    assert!(!db_approved);
}

#[tokio::test]
async fn test_auto_approval_flow() {
    let policies = vec![
        Policy {
            id: "auto-approve-ls".to_string(),
            name: "Auto Approve LS".to_string(),
            description: None,
            resource_type: ResourceType::Command,
            priority: 10,
            allowed_patterns: vec!["ls".to_string()],
            max_ttl_seconds: 0,
            auto_approve: true,
            default_scope: ApprovalScope::Once,
        }
    ];
    let daemon = setup_daemon_with_policies(policies).await;

    let req = Request::new(ExecuteCommandRequest {
        request_id: Uuid::new_v4().to_string(),
        command: "ls".to_string(),
        args: vec![],
        reason: "auto testing".to_string(),
        task_id: None,
        env_vars: HashMap::new(),
        working_dir: None,
        timeout_seconds: 10,
    });
    
    let res = daemon.execute_command(req).await.unwrap().into_inner();
    // Should be EXECUTED immediately without PENDING_APPROVAL
    assert_eq!(res.status, "EXECUTED");
}
