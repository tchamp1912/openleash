use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResourceType {
    #[serde(rename = "secret")]
    Secret,
    #[serde(rename = "package")]
    Package,
    #[serde(rename = "system")]
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageManager {
    #[serde(rename = "pip")]
    Pip,
    #[serde(rename = "npm")]
    Npm,
    #[serde(rename = "brew")]
    Brew,
}

/// Request state tracking (planned for future use).
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestState {
    Received,
    Validated,
    AutoApproved,
    PendingApproval,
    Denied,
    TokenIssued,
    Executed,
    Completed,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApprovalScope {
    /// Only the specific request is approved
    #[serde(rename = "once")]
    Once,
    /// Approval lasts for the duration of the current task
    #[serde(rename = "task")]
    Task,
    /// Approval is permanent for the matched pattern
    #[serde(rename = "permanent")]
    Permanent,
}

impl Default for ApprovalScope {
    fn default() -> Self {
        Self::Once
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub resource_type: ResourceType,
    pub priority: i32,
    pub allowed_patterns: Vec<String>, // Regex patterns for resource_id
    pub max_ttl_seconds: u64,
    pub auto_approve: bool,
    #[serde(default)]
    pub default_scope: ApprovalScope,
}

#[derive(Debug, PartialEq)]
pub enum Decision {
    Allow,
    Deny(String),
    PendingApproval(ApprovalScope),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: Uuid,
    pub request_id: Uuid,
    pub task_id: Option<Uuid>,
    pub resource_type: ResourceType,
    pub resource_id: String,
    pub reason: String,
    pub status: String, // Pending, Approved, Denied
    pub scope: ApprovalScope,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub scope_path: String,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Active,
    Completed,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lease {
    pub id: Uuid,
    pub request_id: Uuid,
    pub task_id: Option<Uuid>,
    pub status: LeaseStatus,
    pub manager: PackageManager,
    pub package_name: String,
    pub package_version: String,
    pub scope_path: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LeaseStatus {
    Active,
    Expired,
    Revoked,
}

/// JWT-style capability tokens for time-bounded, attributed access (planned feature).
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub jti: String,
    pub iat: i64,
    pub nbf: i64,
    pub exp: i64,
    pub resource_type: String,
    pub resource_id: String,
    pub scope_path: String,
    pub operation: String,
    pub policy_id: String,
    pub approval_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub actor: String,
    pub resource_type: ResourceType,
    pub resource_id: String,
    pub action: String,
    pub status: String,
    pub metadata: HashMap<String, String>,
    pub integrity_hash: String,
}

impl AuditEvent {
    pub fn calculate_integrity_hash(&self, previous_hash: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(previous_hash.as_bytes());
        hasher.update(self.id.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        hasher.update(self.event_type.as_bytes());
        hasher.update(self.actor.as_bytes());
        hasher.update(format!("{:?}", self.resource_type).as_bytes());
        hasher.update(self.resource_id.as_bytes());
        hasher.update(self.action.as_bytes());
        hasher.update(self.status.as_bytes());
        
        let hash = hasher.finalize();
        hex::encode(hash)
    }
}
