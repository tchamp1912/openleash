use crate::LeashDaemon;
use leash_ai_api::pb::request_service_server::RequestService;
use leash_ai_api::pb::task_service_server::TaskService;
use leash_ai_api::pb::approval_service_server::ApprovalService;
use leash_ai_api::pb::{ExecuteCommandRequest, ListPendingApprovalsRequest, ApproveRequest, StartTaskRequest};
use leash_ai_core::models::{Policy, ResourceType, ApprovalScope};
use leash_ai_db::db::Db;
use leash_ai_backend_pip::PipBackend;
use leash_ai_backend_brew::BrewBackend;
use leash_ai_backend_npm::NpmBackend;
use leash_ai_backend_keychain::KeychainBackend;
use leash_ai_core::policy::PolicyEngine;
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
            resource_type: ResourceType::Command,
            priority: 10,
            allowed_patterns: vec!["^ls$".to_string()],
            auto_approve: false, 
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

    let inner_req = ExecuteCommandRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        command: "ls".to_string(),
        args: vec![],
        reason: "test".to_string(),
        task_id: None,
        env_vars: std::collections::HashMap::new(),
        working_dir: None,
        timeout_seconds: 5,
    };

    // 1. Initially Pending
    let res = daemon.execute_command(Request::new(inner_req.clone())).await.unwrap().into_inner();
    assert_eq!(res.status, "PENDING_APPROVAL");

    // 2. Approve with Once scope
    let list = daemon.list_pending_approvals(Request::new(ListPendingApprovalsRequest {})).await.unwrap().into_inner();
    let app_id = list.approvals[0].approval_id.clone();
    daemon.approve(Request::new(ApproveRequest { approval_id: app_id, scope: Some("once".to_string()) })).await.unwrap();

    // 3. Success for this request_id
    let res2 = daemon.execute_command(Request::new(inner_req)).await.unwrap().into_inner();
    assert_eq!(res2.status, "EXECUTED");

    // 4. New request_id should be Pending again
    let inner_req_new = ExecuteCommandRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        command: "ls".to_string(),
        args: vec![],
        reason: "test again".to_string(),
        task_id: None,
        env_vars: std::collections::HashMap::new(),
        working_dir: None,
        timeout_seconds: 5,
    };
    let res3 = daemon.execute_command(Request::new(inner_req_new)).await.unwrap().into_inner();
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
            resource_type: ResourceType::Command,
            priority: 10,
            allowed_patterns: vec!["^ls$".to_string()],
            auto_approve: false, 
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

    let inner_req = ExecuteCommandRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        command: "ls".to_string(),
        args: vec![],
        reason: "test".to_string(),
        task_id: None,
        env_vars: std::collections::HashMap::new(),
        working_dir: None,
        timeout_seconds: 5,
    };

    // 1. Approve with Permanent scope
    let _ = daemon.execute_command(Request::new(inner_req.clone())).await.unwrap();
    let list = daemon.list_pending_approvals(Request::new(ListPendingApprovalsRequest {})).await.unwrap().into_inner();
    let app_id = list.approvals[0].approval_id.clone();
    daemon.approve(Request::new(ApproveRequest { approval_id: app_id, scope: Some("permanent".to_string()) })).await.unwrap();

    // 2. Success for ANY request_id
    let inner_req_new = ExecuteCommandRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        command: "ls".to_string(),
        args: vec![],
        reason: "test again".to_string(),
        task_id: None,
        env_vars: std::collections::HashMap::new(),
        working_dir: None,
        timeout_seconds: 5,
    };
    let res = daemon.execute_command(Request::new(inner_req_new)).await.unwrap().into_inner();
    assert_eq!(res.status, "EXECUTED");
}

#[tokio::test]
async fn test_approval_cycle_task() {
    let db = Arc::new(Db::new("sqlite::memory:").await.unwrap());
    let policies = vec![
        Policy {
            id: "require-approval".to_string(),
            name: "Require Approval".to_string(),
            description: None,
            resource_type: ResourceType::Command,
            priority: 10,
            allowed_patterns: vec!["^ls$".to_string()],
            auto_approve: false, 
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
        base_scope_path: "/tmp/leash-task-scope".to_string(),
    })).await.unwrap().into_inner();
    let task_id = start_res.task_id;

    let inner_req = ExecuteCommandRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        command: "ls".to_string(),
        args: vec![],
        reason: "test".to_string(),
        task_id: Some(task_id.clone()),
        env_vars: std::collections::HashMap::new(),
        working_dir: None,
        timeout_seconds: 5,
    };

    // 1. Approve with Task scope
    let _ = daemon.execute_command(Request::new(inner_req.clone())).await.unwrap();
    let list = daemon.list_pending_approvals(Request::new(ListPendingApprovalsRequest {})).await.unwrap().into_inner();
    let app_id = list.approvals[0].approval_id.clone();
    daemon.approve(Request::new(ApproveRequest { approval_id: app_id, scope: Some("task".to_string()) })).await.unwrap();

    // 2. Success for same task, DIFFERENT request_id
    let inner_req_new = ExecuteCommandRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        command: "ls".to_string(),
        args: vec![],
        reason: "test again".to_string(),
        task_id: Some(task_id.clone()),
        env_vars: std::collections::HashMap::new(),
        working_dir: None,
        timeout_seconds: 5,
    };
    let res = daemon.execute_command(Request::new(inner_req_new)).await.unwrap().into_inner();
    assert_eq!(res.status, "EXECUTED");

    // 3. Different task should be Pending (create a second task first)
    let start_res2 = daemon.start_task(Request::new(StartTaskRequest {
        name: "test2".to_string(),
        ttl_seconds: 3600,
        base_scope_path: "/tmp/leash-task-scope-2".to_string(),
    })).await.unwrap().into_inner();
    let task_id2 = start_res2.task_id;
    
    let inner_req_diff_task = ExecuteCommandRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        command: "ls".to_string(),
        args: vec![],
        reason: "different task".to_string(),
        task_id: Some(task_id2),
        env_vars: std::collections::HashMap::new(),
        working_dir: None,
        timeout_seconds: 5,
    };
    let res_diff = daemon.execute_command(Request::new(inner_req_diff_task)).await.unwrap().into_inner();
    assert_eq!(res_diff.status, "PENDING_APPROVAL");
}