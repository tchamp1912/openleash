use crate::models::{Policy, ResourceType, Decision, ApprovalScope};
use crate::policy::PolicyEngine;

#[test]
fn test_policy_evaluation_priority() {
    let policies = vec![
        Policy {
            id: "deny-banned".to_string(),
            name: "Deny Banned".to_string(),
            description: None,
            resource_type: ResourceType::Package,
            priority: 100,
            allowed_patterns: vec!["banned-pkg".to_string()],
            max_ttl_seconds: 3600,
            auto_approve: false,
            default_scope: ApprovalScope::Once,
        },
        Policy {
            id: "allow-all".to_string(),
            name: "Allow All".to_string(),
            description: None,
            resource_type: ResourceType::Package,
            priority: 0,
            allowed_patterns: vec![".*".to_string()],
            max_ttl_seconds: 3600,
            auto_approve: true,
            default_scope: ApprovalScope::Once,
        },
    ];

    let engine = PolicyEngine::new(policies);

    // Test priority (deny-banned has higher priority)
    assert_eq!(engine.evaluate(ResourceType::Package, "banned-pkg"), Decision::PendingApproval(ApprovalScope::Once));
}

#[test]
fn test_approval_scope_inheritance() {
    let policies = vec![
        Policy {
            id: "task-scope-policy".to_string(),
            name: "Task Scope".to_string(),
            description: None,
            resource_type: ResourceType::Command,
            priority: 10,
            allowed_patterns: vec![".*".to_string()],
            max_ttl_seconds: 0,
            auto_approve: false,
            default_scope: ApprovalScope::Task,
        },
    ];

    let engine = PolicyEngine::new(policies);

    assert_eq!(
        engine.evaluate(ResourceType::Command, "some-cmd"), 
        Decision::PendingApproval(ApprovalScope::Task)
    );
}

#[test]
fn test_policy_evaluation_matching() {
    let policies = vec![
        Policy {
            id: "allow-all-pkgs".to_string(),
            name: "Allow All".to_string(),
            description: None,
            resource_type: ResourceType::Package,
            priority: 0,
            allowed_patterns: vec![".*".to_string()],
            max_ttl_seconds: 3600,
            auto_approve: true,
            default_scope: ApprovalScope::Once,
        },
    ];

    let engine = PolicyEngine::new(policies);

    // Test auto-approve
    assert_eq!(engine.evaluate(ResourceType::Package, "any-pkg"), Decision::Allow);

    // Test non-matching resource type
    assert_eq!(
        engine.evaluate(ResourceType::Secret, "any-secret"), 
        Decision::Deny("No matching policy found".to_string())
    );
}

#[test]
fn test_command_policy_regex() {
    let policies = vec![
        Policy {
            id: "allow-ls".to_string(),
            name: "Allow LS".to_string(),
            description: None,
            resource_type: ResourceType::Command,
            priority: 10,
            allowed_patterns: vec!["^ls$".to_string()],
            max_ttl_seconds: 0,
            auto_approve: true,
            default_scope: ApprovalScope::Once,
        },
    ];

    let engine = PolicyEngine::new(policies);

    assert_eq!(engine.evaluate(ResourceType::Command, "ls"), Decision::Allow);
    assert_eq!(engine.evaluate(ResourceType::Command, "ls -la"), Decision::Deny("No matching policy found".to_string()));
}