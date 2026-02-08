use tonic::{Request, Response, Status};
use openleash_api::pb::request_service_server::RequestService;
use openleash_api::pb::{RequestPackageRequest, RequestPackageResponse, RequestSecretRequest, RequestSecretResponse, StoreSecretRequest, StoreSecretResponse};
use openleash_core::models::{ApprovalRequest, ApprovalScope, Lease, LeaseStatus, PackageManager, ResourceType, TaskStatus};
use openleash_backend::{PackageBackend, SecretBackend};
use crate::OpenLeashDaemon;
use crate::daemon::ApprovalHandling;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use std::collections::HashMap;

#[tonic::async_trait]
impl RequestService for OpenLeashDaemon {
    async fn request_package(
        &self,
        request: Request<RequestPackageRequest>,
    ) -> std::result::Result<Response<RequestPackageResponse>, Status> {
        let req = request.into_inner();
        
        let manager = match req.manager.as_str() {
            "pip" => PackageManager::Pip,
            "brew" => PackageManager::Brew,
            "npm" => PackageManager::Npm,
            _ => return Err(Status::invalid_argument("Unsupported package manager")),
        };

        let task_id = req.task_id.as_ref().and_then(|id| Uuid::parse_str(id).ok());

        let final_scope_path = if let Some(tid) = task_id {
            let task = self.db.get_task(&tid)
                .await
                .map_err(|_| Status::failed_precondition("Task not found or inactive"))?;
            
            if task.status != TaskStatus::Active {
                return Err(Status::failed_precondition("Task is no longer active"));
            }
            task.scope_path
        } else {
            req.scope_path.clone()
        };

        // 0. Check for existing approved resources (Task/Permanent)
        if let Ok(true) = self.db.check_approval(ResourceType::Package, &req.package, task_id).await {
            tracing::info!(package = %req.package, "Package installation pre-approved via persistent scope");
        } else {
            let request_id = Uuid::parse_str(&req.request_id).unwrap_or_else(|_| Uuid::new_v4());
            let decision = self.policy_engine.evaluate(ResourceType::Package, &req.package);
            
            match self.handle_approval_decision(
                decision,
                request_id,
                ResourceType::Package,
                req.package.clone(),
                req.reason.clone(),
                task_id,
            ).await? {
                ApprovalHandling::Approved => {
                    // Proceed with installation
                }
                ApprovalHandling::Denied(reason) => {
                    self.audit("PACKAGE", "agent", ResourceType::Package, &req.package, "INSTALL", "DENIED", HashMap::from([("reason".to_string(), reason.clone())])).await?;
                    return Err(Status::permission_denied(reason));
                }
                ApprovalHandling::Pending => {
                    return Ok(Response::new(RequestPackageResponse {
                        request_id: req.request_id,
                        status: "PENDING_APPROVAL".to_string(),
                        lease_id: String::new(),
                        error_message: "Request is pending human approval".to_string(),
                    }));
                }
            }
        }

        let backend: Arc<dyn PackageBackend> = match manager {
            PackageManager::Pip => self.pip_backend.clone(),
            PackageManager::Brew => self.brew_backend.clone(),
            PackageManager::Npm => self.npm_backend.clone(),
        };

        tracing::info!(package = %req.package, manager = %req.manager, task_id = ?task_id, "Installing package");

        let result = backend.install(&req.package, None, &final_scope_path).await;
        
        let status_str = if result.is_ok() { "SUCCESS" } else { "FAILED" };
        let mut metadata = HashMap::from([
            ("manager".to_string(), req.manager.clone()),
            ("reason".to_string(), req.reason.clone()),
        ]);
        if let Some(tid) = req.task_id.clone() {
            metadata.insert("task_id".to_string(), tid);
        }

        self.audit("PACKAGE", "agent", ResourceType::Package, &req.package, "INSTALL", status_str, metadata).await?;

        result.map_err(|e| Status::internal(format!("Installation failed: {}", e)))?;

        let lease = Lease {
            id: Uuid::new_v4(),
            request_id: Uuid::parse_str(&req.request_id).unwrap_or_else(|_| Uuid::new_v4()),
            task_id,
            status: LeaseStatus::Active,
            manager,
            package_name: req.package.clone(),
            package_version: "unknown".to_string(),
            scope_path: final_scope_path,
            expires_at: Utc::now() + chrono::Duration::seconds(req.ttl_seconds as i64),
        };

        self.db.insert_lease(&lease)
            .await
            .map_err(|e| Status::internal(format!("Failed to store lease: {}", e)))?;

        Ok(Response::new(RequestPackageResponse {
            request_id: req.request_id,
            status: "EXECUTED".to_string(),
            lease_id: lease.id.to_string(),
            error_message: String::new(),
        }))
    }

    async fn request_secret(
        &self,
        request: Request<RequestSecretRequest>,
    ) -> std::result::Result<Response<RequestSecretResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(secret_id = %req.secret_id, "Secret requested");

        let task_id = req.task_id.as_ref().and_then(|id| Uuid::parse_str(id).ok());
        if let Some(tid) = task_id {
            let task = self.db.get_task(&tid)
                .await
                .map_err(|_| Status::failed_precondition("Task not found or inactive"))?;
            
            if task.status != TaskStatus::Active {
                return Err(Status::failed_precondition("Task is no longer active"));
            }
        }

        // 1. Check if Keychain is locked
        if let Ok(true) = self.keychain_backend.is_locked().await {
            tracing::warn!("Keychain is locked, requesting unlock approval");
            
            let request_id = Uuid::parse_str(&req.request_id).unwrap_or_else(|_| Uuid::new_v4());
            
            // Check if there is already a pending unlock request
            match self.db.get_approval_by_request_id(&request_id).await {
                Ok(Some(found)) => {
                    if found.status == "Approved" {
                        // Attempt to unlock (using environment variable or prompt)
                        let password = std::env::var("OPENLEASH_KEYCHAIN_PASSWORD").ok();
                        if let Err(e) = self.keychain_backend.unlock(password.as_deref()).await {
                             return Err(Status::internal(format!("Human approved unlock, but unlock failed: {}", e)));
                        }
                        // Now continue to normal evaluation
                    } else if found.status == "Denied" {
                        return Err(Status::permission_denied("Keychain unlock was denied by human approver"));
                    } else {
                        return Ok(Response::new(RequestSecretResponse {
                            request_id: req.request_id,
                            status: "LOCKED".to_string(),
                            value: String::new(),
                            lease_id: String::new(),
                            error_message: "Keychain is locked. Please approve the unlock request on Telegram.".to_string(),
                        }));
                    }
                }
                Ok(None) => {
                    let approval_id = Uuid::new_v4();
                    let approval_req = ApprovalRequest {
                        id: approval_id,
                        request_id,
                        task_id: None,
                        resource_type: ResourceType::System,
                        resource_id: "keychain-unlock".to_string(),
                        reason: "Keychain is locked and must be unlocked to access secrets.".to_string(),
                        status: "Pending".to_string(),
                        scope: ApprovalScope::Once,
                        created_at: Utc::now(),
                        expires_at: Utc::now() + chrono::Duration::seconds(900),
                    };

                    self.db.insert_approval_request(&approval_req).await
                        .map_err(|e| Status::internal(e.to_string()))?;

                    self.notify_approval(&approval_req).await;

                    return Ok(Response::new(RequestSecretResponse {
                        request_id: req.request_id,
                        status: "LOCKED".to_string(),
                        value: String::new(),
                        lease_id: String::new(),
                        error_message: "Keychain is locked. Unlock request sent to Telegram.".to_string(),
                    }));
                }
                Err(e) => return Err(Status::internal(e.to_string())),
            }
        }

        // 1.5 Check for existing approved resources (Task/Permanent)
        if let Ok(true) = self.db.check_approval(ResourceType::Secret, &req.secret_id, task_id).await {
            tracing::info!(secret_id = %req.secret_id, "Secret access pre-approved via persistent scope");
        } else {
            // 2. Normal Policy Evaluation
            let request_id = Uuid::parse_str(&req.request_id).unwrap_or_else(|_| Uuid::new_v4());
            let decision = self.policy_engine.evaluate(ResourceType::Secret, &req.secret_id);
            
            match self.handle_approval_decision(
                decision,
                request_id,
                ResourceType::Secret,
                req.secret_id.clone(),
                req.reason.clone(),
                task_id,
            ).await? {
                ApprovalHandling::Approved => {
                    // Proceed with secret access
                }
                ApprovalHandling::Denied(reason) => {
                    self.audit("SECRET", "agent", ResourceType::Secret, &req.secret_id, "ACCESS", "DENIED", HashMap::from([("reason".to_string(), reason.clone())])).await?;
                    return Err(Status::permission_denied(reason));
                }
                ApprovalHandling::Pending => {
                    return Ok(Response::new(RequestSecretResponse {
                        request_id: req.request_id,
                        status: "PENDING_APPROVAL".to_string(),
                        value: String::new(),
                        lease_id: String::new(),
                        error_message: "Request is pending human approval".to_string(),
                    }));
                }
            }
        }

        let result = self.keychain_backend.get_secret(&req.secret_id).await;
        
        let status_str = if result.is_ok() { "GRANTED" } else { "DENIED" };
        let mut metadata = HashMap::from([
            ("reason".to_string(), req.reason.clone()),
        ]);
        if let Some(tid) = req.task_id.clone() {
            metadata.insert("task_id".to_string(), tid);
        }

        self.audit("SECRET", "agent", ResourceType::Secret, &req.secret_id, "ACCESS", status_str, metadata).await?;

        match result {
            Ok(value) => {
                let lease_id = Uuid::new_v4();
                Ok(Response::new(RequestSecretResponse {
                    request_id: req.request_id,
                    status: "GRANTED".to_string(),
                    value,
                    lease_id: lease_id.to_string(),
                    error_message: String::new(),
                }))
            }
            Err(e) => {
                tracing::warn!(secret_id = %req.secret_id, error = %e, "Secret access denied");
                Ok(Response::new(RequestSecretResponse {
                    request_id: req.request_id,
                    status: "DENIED".to_string(),
                    value: String::new(),
                    lease_id: String::new(),
                    error_message: e.to_string(),
                }))
            }
        }
    }

    async fn store_secret(
        &self,
        request: Request<StoreSecretRequest>,
    ) -> std::result::Result<Response<StoreSecretResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(secret_id = %req.secret_id, "Secret store request");

        let task_id = req.task_id.as_ref().and_then(|id| Uuid::parse_str(id).ok());
        if let Some(tid) = task_id {
            let task = self.db.get_task(&tid)
                .await
                .map_err(|_| Status::failed_precondition("Task not found or inactive"))?;
            
            if task.status != TaskStatus::Active {
                return Err(Status::failed_precondition("Task is no longer active"));
            }
        }

        // 0. Check for existing approved resources (Task/Permanent)
        if let Ok(true) = self.db.check_approval(ResourceType::Secret, &req.secret_id, task_id).await {
            tracing::info!(secret_id = %req.secret_id, "Secret store pre-approved via persistent scope");
        } else {
            let request_id = Uuid::parse_str(&req.request_id).unwrap_or_else(|_| Uuid::new_v4());
            let decision = self.policy_engine.evaluate(ResourceType::Secret, &req.secret_id);
            
            match self.handle_approval_decision(
                decision,
                request_id,
                ResourceType::Secret,
                req.secret_id.clone(),
                req.reason.clone(),
                task_id,
            ).await? {
                ApprovalHandling::Approved => {
                    // Proceed with secret storage
                }
                ApprovalHandling::Denied(reason) => {
                    self.audit("SECRET", "agent", ResourceType::Secret, &req.secret_id, "STORE", "DENIED", HashMap::from([("reason".to_string(), reason.clone())])).await?;
                    return Err(Status::permission_denied(reason));
                }
                ApprovalHandling::Pending => {
                    return Ok(Response::new(StoreSecretResponse {
                        request_id: req.request_id,
                        success: false,
                        error_message: "Request is pending human approval".to_string(),
                    }));
                }
            }
        }

        let result = self.keychain_backend.store_secret(&req.secret_id, &req.value).await;
        
        let status_str = if result.is_ok() { "SUCCESS" } else { "FAILED" };
        let mut metadata = HashMap::from([
            ("reason".to_string(), req.reason.clone()),
        ]);
        if let Some(tid) = req.task_id.clone() {
            metadata.insert("task_id".to_string(), tid);
        }

        self.audit("SECRET", "agent", ResourceType::Secret, &req.secret_id, "STORE", status_str, metadata).await?;

        match result {
            Ok(_) => {
                Ok(Response::new(StoreSecretResponse {
                    request_id: req.request_id,
                    success: true,
                    error_message: String::new(),
                }))
            }
            Err(e) => {
                tracing::error!(secret_id = %req.secret_id, error = %e, "Failed to store secret");
                Ok(Response::new(StoreSecretResponse {
                    request_id: req.request_id,
                    success: false,
                    error_message: e.to_string(),
                }))
            }
        }
    }
}
