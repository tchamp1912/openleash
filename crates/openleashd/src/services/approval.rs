use tonic::{Request, Response, Status};
use openleash_api::pb::approval_service_server::ApprovalService;
use openleash_api::pb::{ListPendingApprovalsRequest, ListPendingApprovalsResponse, ApproveRequest, ApproveResponse, DenyRequest, DenyResponse, PendingApproval};
use crate::OpenLeashDaemon;
use uuid::Uuid;

#[tonic::async_trait]
impl ApprovalService for OpenLeashDaemon {
    async fn list_pending_approvals(
        &self,
        _request: Request<ListPendingApprovalsRequest>,
    ) -> std::result::Result<Response<ListPendingApprovalsResponse>, Status> {
        let pending = self.db.get_pending_approvals().await
            .map_err(|e| Status::internal(e.to_string()))?;
        
        let approvals = pending.into_iter().map(|a| PendingApproval {
            approval_id: a.id.to_string(),
            request_id: a.request_id.to_string(),
            resource_type: format!("{:?}", a.resource_type),
            resource_id: a.resource_id,
            reason: a.reason,
            created_at: Some(prost_types::Timestamp {
                seconds: a.created_at.timestamp(),
                nanos: a.created_at.timestamp_subsec_nanos() as i32,
            }),
        }).collect();

        Ok(Response::new(ListPendingApprovalsResponse { approvals }))
    }

    async fn approve(
        &self,
        request: Request<ApproveRequest>,
    ) -> std::result::Result<Response<ApproveResponse>, Status> {
        let req = request.into_inner();
        let approval_id = Uuid::parse_str(&req.approval_id)
            .map_err(|_| Status::invalid_argument("Invalid approval_id"))?;

        let override_scope = req.scope.and_then(|s| match s.to_lowercase().as_str() {
            "once" => Some(openleash_core::models::ApprovalScope::Once),
            "task" => Some(openleash_core::models::ApprovalScope::Task),
            "permanent" => Some(openleash_core::models::ApprovalScope::Permanent),
            _ => None,
        });

        self.db.update_approval_status(&approval_id, "Approved", override_scope).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ApproveResponse { success: true }))
    }

    async fn deny(
        &self,
        request: Request<DenyRequest>,
    ) -> std::result::Result<Response<DenyResponse>, Status> {
        let req = request.into_inner();
        let approval_id = Uuid::parse_str(&req.approval_id)
            .map_err(|_| Status::invalid_argument("Invalid approval_id"))?;

        self.db.update_approval_status(&approval_id, "Denied", None).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DenyResponse { success: true }))
    }
}
