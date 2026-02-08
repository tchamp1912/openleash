use tonic::{Request, Response, Status};
use openleash_api::pb::audit_service_server::AuditService;
use openleash_api::pb::{QueryAuditLogsRequest, QueryAuditLogsResponse, AuditEntry};
use crate::OpenLeashDaemon;
use chrono::{Utc, TimeZone};

#[tonic::async_trait]
impl AuditService for OpenLeashDaemon {
    async fn query_audit_logs(
        &self,
        request: Request<QueryAuditLogsRequest>,
    ) -> std::result::Result<Response<QueryAuditLogsResponse>, Status> {
        let req = request.into_inner();
        
        let start_time = req.start_time.and_then(|t| {
            Utc.timestamp_opt(t.seconds, t.nanos as u32).latest()
        });
        
        let end_time = req.end_time.and_then(|t| {
            Utc.timestamp_opt(t.seconds, t.nanos as u32).latest()
        });

        let logs = self.db.query_audit_logs(start_time, end_time, req.limit).await
            .map_err(|e| Status::internal(e.to_string()))?;

        let entries = logs.into_iter().map(|l| AuditEntry {
            id: l.id.to_string(),
            timestamp: Some(prost_types::Timestamp {
                seconds: l.timestamp.timestamp(),
                nanos: l.timestamp.timestamp_subsec_nanos() as i32,
            }),
            event_type: l.event_type,
            actor: l.actor,
            resource_type: format!("{:?}", l.resource_type),
            resource_id: l.resource_id,
            action: l.action,
            status: l.status,
            integrity_hash: l.integrity_hash,
        }).collect();

        Ok(Response::new(QueryAuditLogsResponse { entries }))
    }
}
