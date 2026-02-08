use crate::OpenLeashDaemon;
use openleash_api::pb::request_service_server::RequestService;
use openleash_api::pb::task_service_server::TaskService;
use openleash_api::pb::approval_service_server::ApprovalService;
use openleash_api::pb::{ListPendingApprovalsRequest, ApproveRequest, RequestSecretRequest, StartTaskRequest};
use openleash_core::models::{Policy, ResourceType, ApprovalScope};
use openleash_db::db::Db;
use openleash_backend_pip::PipBackend;
use openleash_backend_brew::BrewBackend;
use openleash_backend_npm::NpmBackend;
use openleash_backend_keychain::KeychainBackend;
use openleash_core::policy::PolicyEngine;
use std::sync::Arc;
use tonic::Request;

#[tokio::test]
async fn test_full_approval_cycle_once() {
    let db = Arc::new(Db::new("sqlite::memory:").await.unwrap());
    let policies = vec![
        Policy {
            id: "require-approval".to_string(),
            name: "Require Approval".to_string(),
            description: None,
            resource_type: ResourceType::Secret,
            priority: 10,
            allowed_patterns: vec![".*".to_string()],
            auto_approve: false, 
            max_ttl_seconds: 0,
            default_scope: ApprovalScope::Once,
        }
    ];
    let daemon = Arc::new(OpenLeashDaemon {
        db,
        pip_backend: Arc::new(PipBackend),
        brew_backend: Arc::new(BrewBackend),
        npm_backend: Arc::new(NpmBackend::default()),
        keychain_backend: Arc::new(KeychainBackend),
        policy_engine: Arc::new(PolicyEngine::new(policies)),
        approval_backends: vec![],
    });

    let inner_req = RequestSecretRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        secret_id: "test-secret".to_string(),
        reason: "test".to_string(),
        task_id: None,
    };

    // 1. Initially Pending
    let res = daemon.request_secret(Request::new(inner_req.clone())).await.unwrap().into_inner();
    assert_eq!(res.status, "PENDING_APPROVAL");

    // 2. Approve with Once scope
    let list = daemon.list_pending_approvals(Request::new(ListPendingApprovalsRequest {})).await.unwrap().into_inner();
    let app_id = list.approvals[0].approval_id.clone();
    daemon.approve(Request::new(ApproveRequest { approval_id: app_id, scope: Some("once".to_string()) })).await.unwrap();

    // 3. Success for this request_id
    let res2 = daemon.request_secret(Request::new(inner_req)).await.unwrap().into_inner();
    assert_ne!(res2.status, "PENDING_APPROVAL");

    // 4. New request_id should be Pending again
    let inner_req_new = RequestSecretRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        secret_id: "test-secret".to_string(),
        reason: "test again".to_string(),
        task_id: None,
    };
    let res3 = daemon.request_secret(Request::new(inner_req_new)).await.unwrap().into_inner();
    assert_eq!(res3.status, "PENDING_APPROVAL");
}

#[tokio::test]
async fn test_approval_cycle_permanent() {
    let db = Arc::new(Db::new("sqlite::memory:").await.unwrap());
    let policies = vec![
        Policy {
            id: "require-approval".to_string(),
            name: "Require Approval".to_string(),
            description: None,
            resource_type: ResourceType::Secret,
            priority: 10,
            allowed_patterns: vec![".*".to_string()],
            auto_approve: false, 
            max_ttl_seconds: 0,
            default_scope: ApprovalScope::Once,
        }
    ];
    let daemon = Arc::new(OpenLeashDaemon {
        db,
        pip_backend: Arc::new(PipBackend),
        brew_backend: Arc::new(BrewBackend),
        npm_backend: Arc::new(NpmBackend::default()),
        keychain_backend: Arc::new(KeychainBackend),
        policy_engine: Arc::new(PolicyEngine::new(policies)),
        approval_backends: vec![],
    });

    let inner_req = RequestSecretRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        secret_id: "test-secret".to_string(),
        reason: "test".to_string(),
        task_id: None,
    };

    // 1. Approve with Permanent scope
    let _ = daemon.request_secret(Request::new(inner_req.clone())).await.unwrap();
    let list = daemon.list_pending_approvals(Request::new(ListPendingApprovalsRequest {})).await.unwrap().into_inner();
    let app_id = list.approvals[0].approval_id.clone();
    daemon.approve(Request::new(ApproveRequest { approval_id: app_id, scope: Some("permanent".to_string()) })).await.unwrap();

    // 2. Success for ANY request_id
    let inner_req_new = RequestSecretRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        secret_id: "test-secret".to_string(),
        reason: "test again".to_string(),
        task_id: None,
    };
    let res = daemon.request_secret(Request::new(inner_req_new)).await.unwrap().into_inner();
    assert_ne!(res.status, "PENDING_APPROVAL");
}

#[tokio::test]
async fn test_approval_cycle_task() {
    let db = Arc::new(Db::new("sqlite::memory:").await.unwrap());
    let policies = vec![
        Policy {
            id: "require-approval".to_string(),
            name: "Require Approval".to_string(),
            description: None,
            resource_type: ResourceType::Secret,
            priority: 10,
            allowed_patterns: vec![".*".to_string()],
            auto_approve: false, 
            max_ttl_seconds: 0,
            default_scope: ApprovalScope::Once,
        }
    ];
    let daemon = Arc::new(OpenLeashDaemon {
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
        base_scope_path: "/tmp/leash-task-scope".to_string(),
    })).await.unwrap().into_inner();
    let task_id = start_res.task_id;

    let inner_req = RequestSecretRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        secret_id: "test-secret".to_string(),
        reason: "test".to_string(),
        task_id: Some(task_id.clone()),
    };

    // 1. Approve with Task scope
    let _ = daemon.request_secret(Request::new(inner_req.clone())).await.unwrap();
    let list = daemon.list_pending_approvals(Request::new(ListPendingApprovalsRequest {})).await.unwrap().into_inner();
    let app_id = list.approvals[0].approval_id.clone();
    daemon.approve(Request::new(ApproveRequest { approval_id: app_id, scope: Some("task".to_string()) })).await.unwrap();

    // 2. Success for same task, DIFFERENT request_id
    let inner_req_new = RequestSecretRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        secret_id: "test-secret".to_string(),
        reason: "test again".to_string(),
        task_id: Some(task_id.clone()),
    };
    let res = daemon.request_secret(Request::new(inner_req_new)).await.unwrap().into_inner();
    assert_ne!(res.status, "PENDING_APPROVAL");

    // 3. Different task should be Pending (create a second task first)
    let start_res2 = daemon.start_task(Request::new(StartTaskRequest {
        name: "test2".to_string(),
        ttl_seconds: 3600,
        base_scope_path: "/tmp/leash-task-scope-2".to_string(),
    })).await.unwrap().into_inner();
    let task_id2 = start_res2.task_id;
    
    let inner_req_diff_task = RequestSecretRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        secret_id: "test-secret".to_string(),
        reason: "different task".to_string(),
        task_id: Some(task_id2),
    };
    let res_diff = daemon.request_secret(Request::new(inner_req_diff_task)).await.unwrap().into_inner();
    assert_eq!(res_diff.status, "PENDING_APPROVAL");
}