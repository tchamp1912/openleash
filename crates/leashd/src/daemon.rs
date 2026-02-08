use leash_ai_core::models::{Lease, LeaseStatus, PackageManager, TaskStatus, AuditEvent, ResourceType, ApprovalRequest, Decision};
use leash_ai_core::policy::PolicyEngine;
use leash_ai_db::db::Db;
use leash_ai_backend_pip::PipBackend;
use leash_ai_backend_brew::BrewBackend;
use leash_ai_backend_npm::NpmBackend;
use leash_ai_backend_keychain::KeychainBackend;
use leash_ai_backend::{PackageBackend, ApprovalBackend};
use leash_ai_venv::VenvManager;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use std::collections::HashMap;
use tonic::Status;

/// Result of handling a pending approval decision.
/// Indicates what action the caller should take.
#[derive(Debug)]
pub enum ApprovalHandling {
    /// Approval was granted, proceed with the request
    Approved,
    /// Approval request was created and sent to approvers
    Pending,
    /// Request was denied
    Denied(String),
}

pub struct LeashDaemon {
    pub db: Arc<Db>,
    pub pip_backend: Arc<PipBackend>,
    pub brew_backend: Arc<BrewBackend>,
    pub npm_backend: Arc<NpmBackend>,
    pub keychain_backend: Arc<KeychainBackend>,
    pub policy_engine: Arc<PolicyEngine>,
    pub approval_backends: Vec<Arc<dyn ApprovalBackend>>,
}

impl LeashDaemon {
    pub fn new(
        db: Arc<Db>,
        pip_backend: Arc<PipBackend>,
        brew_backend: Arc<BrewBackend>,
        npm_backend: Arc<NpmBackend>,
        keychain_backend: Arc<KeychainBackend>,
        policy_engine: Arc<PolicyEngine>,
        approval_backends: Vec<Arc<dyn ApprovalBackend>>,
    ) -> Self {
        Self {
            db,
            pip_backend,
            brew_backend,
            npm_backend,
            keychain_backend,
            policy_engine,
            approval_backends,
        }
    }

    pub fn db(&self) -> Arc<Db> {
        self.db.clone()
    }

    pub async fn audit(
        &self,
        event_type: &str,
        actor: &str,
        resource_type: ResourceType,
        resource_id: &str,
        action: &str,
        status: &str,
        metadata: HashMap<String, String>,
    ) -> std::result::Result<(), Status> {
        let previous_hash = self.db.get_last_integrity_hash().await
            .map_err(|e| Status::internal(format!("Audit failed: {}", e)))?
            .unwrap_or_else(|| "START".to_string());

        let mut event = AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: event_type.to_string(),
            actor: actor.to_string(),
            resource_type,
            resource_id: resource_id.to_string(),
            action: action.to_string(),
            status: status.to_string(),
            metadata,
            integrity_hash: String::new(),
        };

        event.integrity_hash = event.calculate_integrity_hash(&previous_hash);

        self.db.insert_audit_event(&event).await
            .map_err(|e| Status::internal(format!("Audit failed: {}", e)))?;

        Ok(())
    }

    pub async fn notify_approval(&self, req: &ApprovalRequest) {
        for backend in &self.approval_backends {
            if let Err(e) = backend.notify_approval(req).await {
                tracing::error!(error = %e, "Failed to notify approval backend");
            }
        }
    }

    pub async fn cleanup_task(&self, task_id: &Uuid, status: TaskStatus) -> std::result::Result<(), Status> {
        let task = self.db.get_task(task_id)
            .await
            .map_err(|e| Status::not_found(format!("Task not found: {}", e)))?;

        let venv = VenvManager::new(&task.scope_path);
        if let Err(e) = venv.remove().await {
            tracing::error!(task_id = %task_id, error = %e, "Failed to cleanup task venv");
        }

        let leases = self.db.get_leases_by_task(task_id)
            .await
            .map_err(|e| Status::internal(format!("Failed to fetch task leases: {}", e)))?;
        
        for lease in leases {
            if let Err(e) = self.db.update_lease_status(&lease.id, LeaseStatus::Expired).await {
                tracing::error!(lease_id = %lease.id, error = %e, "Failed to update lease status");
            }
        }

        if let Err(e) = self.db.delete_approvals_by_task(task_id).await {
            tracing::error!(task_id = %task_id, error = %e, "Failed to cleanup task approvals");
        }

        self.db.update_task_status(task_id, status)
            .await
            .map_err(|e| Status::internal(format!("Failed to update task status: {}", e)))?;

        Ok(())
    }

    pub async fn cleanup_lease(&self, lease: &Lease) -> std::result::Result<(), Status> {
        tracing::info!(lease_id = %lease.id, package = %lease.package_name, "Cleaning up expired standalone lease");
        
        let backend: Arc<dyn PackageBackend> = match lease.manager {
            PackageManager::Pip => self.pip_backend.clone(),
            PackageManager::Brew => self.brew_backend.clone(),
            PackageManager::Npm => self.npm_backend.clone(),
        };

        if let Err(e) = backend.uninstall(&lease.scope_path).await {
            tracing::error!(lease_id = %lease.id, error = %e, "Failed to uninstall package for lease");
        }

        self.db.update_lease_status(&lease.id, LeaseStatus::Expired).await
            .map_err(|e| Status::internal(format!("Failed to update lease status: {}", e)))?;

        Ok(())
    }

    /// Handle a policy decision that requires approval.
    /// Checks for existing approval, creates new request if needed, and notifies approvers.
    /// Returns `ApprovalHandling` indicating what the caller should do next.
    pub async fn handle_approval_decision(
        &self,
        decision: Decision,
        request_id: Uuid,
        resource_type: ResourceType,
        resource_id: String,
        reason: String,
        task_id: Option<Uuid>,
    ) -> std::result::Result<ApprovalHandling, Status> {
        match decision {
            Decision::Allow => Ok(ApprovalHandling::Approved),
            Decision::Deny(reason) => Ok(ApprovalHandling::Denied(reason)),
            Decision::PendingApproval(scope) => {
                // Check if approval already exists for this request_id
                match self.db.get_approval_by_request_id(&request_id).await {
                    Ok(Some(found)) => {
                        match found.status.as_str() {
                            "Approved" => Ok(ApprovalHandling::Approved),
                            "Denied" => Ok(ApprovalHandling::Denied("Request was denied by human approver".to_string())),
                            _ => Ok(ApprovalHandling::Pending),
                        }
                    }
                    Ok(None) => {
                        // Create new approval request
                        let approval_id = Uuid::new_v4();
                        let approval_req = ApprovalRequest {
                            id: approval_id,
                            request_id,
                            task_id,
                            resource_type,
                            resource_id,
                            reason,
                            status: "Pending".to_string(),
                            scope,
                            created_at: Utc::now(),
                            expires_at: Utc::now() + chrono::Duration::seconds(900),
                        };

                        self.db.insert_approval_request(&approval_req).await
                            .map_err(|e| Status::internal(e.to_string()))?;

                        self.notify_approval(&approval_req).await;

                        Ok(ApprovalHandling::Pending)
                    }
                    Err(e) => Err(Status::internal(e.to_string())),
                }
            }
        }
    }
}
