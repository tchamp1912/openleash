use leash_ai_core::models::{Policy, ResourceType, ApprovalScope};

pub fn get_default_policies() -> Vec<Policy> {
    vec![
        Policy {
            id: "default-packages".to_string(),
            name: "Default Packages".to_string(),
            description: Some("Allow all packages".to_string()),
            resource_type: ResourceType::Package,
            priority: 0,
            allowed_patterns: vec![".*".to_string()],
            max_ttl_seconds: 86400,
            auto_approve: true,
            default_scope: ApprovalScope::Once,
        },
        Policy {
            id: "default-secrets".to_string(),
            name: "Default Secrets".to_string(),
            description: Some("Allow all secrets".to_string()),
            resource_type: ResourceType::Secret,
            priority: 0,
            allowed_patterns: vec![".*".to_string()],
            max_ttl_seconds: 0,
            auto_approve: true,
            default_scope: ApprovalScope::Once,
        },
        Policy {
            id: "default-commands".to_string(),
            name: "Default Commands".to_string(),
            description: Some("Allow common safe commands".to_string()),
            resource_type: ResourceType::Command,
            priority: 0,
            allowed_patterns: vec!["^ls$".to_string(), "^cat$".to_string(), "^grep$".to_string(), "^echo$".to_string(), "^sleep$".to_string(), "^python3$".to_string()],
            max_ttl_seconds: 0,
            auto_approve: true,
            default_scope: ApprovalScope::Once,
        }
    ]
}
