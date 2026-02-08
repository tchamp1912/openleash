use crate::models::{Policy, ResourceType, Decision};
use regex::Regex;

pub struct PolicyEngine {
    policies: Vec<Policy>,
}

impl PolicyEngine {
    pub fn new(mut policies: Vec<Policy>) -> Self {
        // Sort by priority (higher first)
        policies.sort_by(|a, b| b.priority.cmp(&a.priority));
        Self { policies }
    }

    pub fn evaluate(&self, resource_type: ResourceType, resource_id: &str) -> Decision {
        for policy in &self.policies {
            if policy.resource_type != resource_type {
                continue;
            }

            for pattern in &policy.allowed_patterns {
                if let Ok(re) = Regex::new(pattern) {
                    if re.is_match(resource_id) {
                        if policy.auto_approve {
                            return Decision::Allow;
                        } else {
                            return Decision::PendingApproval(policy.default_scope.clone());
                        }
                    }
                }
            }
        }

        Decision::Deny("No matching policy found".to_string())
    }
}
