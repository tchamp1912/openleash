use crate::OpenLeashDaemon;
use openleash_api::pb::request_service_server::RequestService;
use openleash_api::pb::approval_service_server::ApprovalService;
use openleash_api::pb::task_service_server::TaskService;
use openleash_api::pb::{RequestSecretRequest, ListPendingApprovalsRequest, ApproveRequest, StartTaskRequest, GetTaskEnvironmentRequest};
use openleash_core::models::{Policy, ResourceType, ApprovalScope};
use openleash_core::policy::PolicyEngine;
use openleash_db::db::Db;
use openleash_backend_pip::PipBackend;
use openleash_backend_brew::BrewBackend;
use openleash_backend_npm::NpmBackend;
use openleash_backend_keychain::KeychainBackend;
use std::sync::Arc;
use tonic::Request;
use uuid::Uuid;

async fn setup_daemon_with_policies(policies: Vec<Policy>) -> Arc<OpenLeashDaemon> {
    let db = Arc::new(Db::new("sqlite::memory:").await.unwrap());
    let policy_engine = Arc::new(PolicyEngine::new(policies));
    
    Arc::new(OpenLeashDaemon {
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
async fn test_get_task_environment() {
    let daemon = setup_daemon_with_policies(vec![]).await;

    // 1. Start a task
    let task_res = daemon.start_task(Request::new(StartTaskRequest {
        name: "test-task".to_string(),
        ttl_seconds: 3600,
        base_scope_path: "/tmp".to_string(),
    })).await.unwrap().into_inner();
    let task_id = task_res.task_id;
    let scope_path = task_res.scope_path.clone();

    // 2. Get task environment
    let env_res = daemon.get_task_environment(Request::new(GetTaskEnvironmentRequest {
        task_id: task_id.clone(),
    })).await.unwrap().into_inner();

    // 3. Verify paths are correct
    assert_eq!(env_res.scope_path, scope_path);
    assert!(env_res.bin_path.contains(&task_id));
    assert!(env_res.bin_path.ends_with("/bin"));
    assert!(env_res.error_message.is_empty());

    // 4. Test with invalid task_id
    let invalid_res = daemon.get_task_environment(Request::new(GetTaskEnvironmentRequest {
        task_id: "invalid-uuid".to_string(),
    })).await;
    assert!(invalid_res.is_err());

    // 5. Test with non-existent task_id
    let nonexistent_res = daemon.get_task_environment(Request::new(GetTaskEnvironmentRequest {
        task_id: Uuid::new_v4().to_string(),
    })).await;
    assert!(nonexistent_res.is_err());

    // 6. End task and verify it fails
    daemon.end_task(Request::new(openleash_api::pb::EndTaskRequest {
        task_id: task_id.clone(),
    })).await.unwrap();

    let ended_res = daemon.get_task_environment(Request::new(GetTaskEnvironmentRequest {
        task_id: task_id.clone(),
    })).await;
    assert!(ended_res.is_err());
    assert!(ended_res.unwrap_err().message().contains("no longer active"));
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

    daemon.deny(Request::new(openleash_api::pb::DenyRequest {
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
    let req = Request::new(openleash_api::pb::RequestPackageRequest {
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
    let req2 = Request::new(openleash_api::pb::RequestPackageRequest {
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

    let req = Request::new(openleash_api::pb::StoreSecretRequest {
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
    let req2 = Request::new(openleash_api::pb::StoreSecretRequest {
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

