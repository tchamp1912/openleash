use leash_ai_api::pb::request_service_client::RequestServiceClient;
use leash_ai_api::pb::task_service_client::TaskServiceClient;
use leash_ai_api::pb::approval_service_client::ApprovalServiceClient;
use leash_ai_api::pb::audit_service_client::AuditServiceClient;
use leash_ai_api::pb::{RequestPackageRequest, StartTaskRequest, EndTaskRequest, GetTaskEnvironmentRequest, ListPendingApprovalsRequest, ApproveRequest, DenyRequest, PendingApproval, QueryAuditLogsRequest, AuditEntry};
use leash_ai_core::{Result, LeashError};
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;
use tokio::net::UnixStream;

pub struct LeashClient {
    request_client: RequestServiceClient<Channel>,
    task_client: TaskServiceClient<Channel>,
    approval_client: ApprovalServiceClient<Channel>,
    audit_client: AuditServiceClient<Channel>,
}

impl LeashClient {
    /// Connect using default settings or environment variables.
    /// Prioritizes LEASH_SERVER or LEASH_DAEMON_URL environment variables.
    /// Defaults to unix:///tmp/leash.sock.
    pub async fn connect_default() -> Result<Self> {
        let dst = std::env::var("LEASH_SERVER")
            .or_else(|_| std::env::var("LEASH_DAEMON_URL"))
            .unwrap_or_else(|_| "unix:///tmp/leash.sock".to_string());
        
        Self::connect(dst).await
    }

    pub async fn connect(dst: String) -> Result<Self> {
        let channel = if dst.starts_with("unix://") {
            let path = dst.replace("unix://", "");
            let path_clone = path.clone();
            
            Endpoint::try_from("http://localhost") // Dummy URI for tonic
                .map_err(|e| LeashError::Internal(e.to_string()))?
                .connect_with_connector(service_fn(move |_: Uri| {
                    let path = path_clone.clone();
                    async move {
                        UnixStream::connect(path).await
                    }
                }))
                .await
                .map_err(|e| LeashError::Internal(format!("Failed to connect to UDS {}: {}", path, e)))?
        } else {
            Channel::from_shared(dst)
                .map_err(|e| LeashError::Internal(e.to_string()))?
                .connect()
                .await
                .map_err(|e| LeashError::Internal(e.to_string()))?
        };

        Ok(Self {
            request_client: RequestServiceClient::new(channel.clone()),
            task_client: TaskServiceClient::new(channel.clone()),
            approval_client: ApprovalServiceClient::new(channel.clone()),
            audit_client: AuditServiceClient::new(channel),
        })
    }

    pub async fn start_task(&mut self, name: &str, base_path: &str, ttl: u64) -> Result<(String, String)> {
        let response = self.task_client.start_task(StartTaskRequest {
            name: name.to_string(),
            base_scope_path: base_path.to_string(),
            ttl_seconds: ttl,
        }).await.map_err(|e| LeashError::Internal(e.to_string()))?;
        
        let res = response.into_inner();
        Ok((res.task_id, res.scope_path))
    }

    pub async fn end_task(&mut self, task_id: &str) -> Result<()> {
        self.task_client.end_task(EndTaskRequest {
            task_id: task_id.to_string(),
        }).await.map_err(|e| LeashError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn get_task_environment(&mut self, task_id: &str) -> Result<(String, String)> {
        let response = self.task_client.get_task_environment(GetTaskEnvironmentRequest {
            task_id: task_id.to_string(),
        }).await.map_err(|e| LeashError::Internal(e.to_string()))?;
        
        let res = response.into_inner();
        if !res.error_message.is_empty() {
            return Err(LeashError::Backend(res.error_message));
        }
        Ok((res.bin_path, res.scope_path))
    }

    pub async fn request_secret(
        &mut self,
        secret_id: &str,
        reason: &str,
        task_id: Option<String>,
    ) -> Result<String> {
        let response = self.request_client.request_secret(tonic::Request::new(leash_ai_api::pb::RequestSecretRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            secret_id: secret_id.to_string(),
            reason: reason.to_string(),
            task_id,
        })).await.map_err(|e| LeashError::Internal(e.to_string()))?;

        let res = response.into_inner();
        if res.status == "GRANTED" {
            Ok(res.value)
        } else {
            Err(LeashError::PermissionDenied(res.error_message))
        }
    }

    pub async fn store_secret(
        &mut self,
        secret_id: &str,
        value: &str,
        reason: &str,
        task_id: Option<String>,
    ) -> Result<()> {
        let response = self.request_client.store_secret(tonic::Request::new(leash_ai_api::pb::StoreSecretRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            secret_id: secret_id.to_string(),
            value: value.to_string(),
            reason: reason.to_string(),
            task_id,
        })).await.map_err(|e| LeashError::Internal(e.to_string()))?;

        let res = response.into_inner();
        if res.success {
            Ok(())
        } else {
            Err(LeashError::Backend(res.error_message))
        }
    }

    pub async fn request_package(
        &mut self,
        manager: &str,
        package: &str,
        scope_path: &str,
        reason: &str,
        ttl_seconds: u64,
        task_id: Option<String>,
    ) -> Result<String> {
        let request = tonic::Request::new(RequestPackageRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            manager: manager.to_string(),
            package: package.to_string(),
            reason: reason.to_string(),
            ttl_seconds,
            scope_path: scope_path.to_string(),
            task_id,
        });

        let response = self.request_client.request_package(request)
            .await
            .map_err(|e| LeashError::Internal(e.to_string()))?;

        let res = response.into_inner();
        if res.status == "EXECUTED" || res.status == "AUTO_APPROVED" {
            Ok(res.lease_id)
        } else {
            Err(LeashError::Backend(res.error_message))
        }
    }

    pub async fn list_pending_approvals(&mut self) -> Result<Vec<PendingApproval>> {
        let response = self.approval_client.list_pending_approvals(ListPendingApprovalsRequest {})
            .await
            .map_err(|e| LeashError::Internal(e.to_string()))?;
        
        Ok(response.into_inner().approvals)
    }

    pub async fn approve(&mut self, approval_id: &str, scope: Option<String>) -> Result<bool> {
        let response = self.approval_client.approve(ApproveRequest {
            approval_id: approval_id.to_string(),
            scope,
        }).await.map_err(|e| LeashError::Internal(e.to_string()))?;
        
        Ok(response.into_inner().success)
    }

    pub async fn deny(&mut self, approval_id: &str) -> Result<bool> {
        let response = self.approval_client.deny(DenyRequest {
            approval_id: approval_id.to_string(),
        }).await.map_err(|e| LeashError::Internal(e.to_string()))?;
        
        Ok(response.into_inner().success)
    }

    pub async fn query_audit_logs(&mut self, limit: u32) -> Result<Vec<AuditEntry>> {
        let response = self.audit_client.query_audit_logs(QueryAuditLogsRequest {
            limit,
            ..Default::default()
        }).await.map_err(|e| LeashError::Internal(e.to_string()))?;
        
        Ok(response.into_inner().entries)
    }
}
