use openleash_api::pb::audit_service_server::AuditService;
use openleash_api::pb::QueryAuditLogsRequest;
use openleash_core::models::{AuditEvent, ResourceType};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;
use tonic::Request;
use crate::tests::test_utils::setup_daemon;

#[tokio::test]
async fn test_audit_log_query() {
    let daemon = setup_daemon().await;
    
    // 1. Insert dummy event via the DB directly
    let event = AuditEvent {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        event_type: "TEST".to_string(),
        actor: "agent".to_string(),
        resource_type: ResourceType::Package,
        resource_id: "test-pkg".to_string(),
        action: "INSTALL".to_string(),
        status: "SUCCESS".to_string(),
        metadata: HashMap::new(),
        integrity_hash: "dummyhash".to_string(),
    };
    
    daemon.db().insert_audit_event(&event).await.unwrap();

    // 2. Query via the service
    let req = Request::new(QueryAuditLogsRequest {
        limit: 10,
        ..Default::default()
    });

    let res = daemon.query_audit_logs(req).await.unwrap().into_inner();
    assert_eq!(res.entries.len(), 1);
    assert_eq!(res.entries[0].resource_id, "test-pkg");
    assert_eq!(res.entries[0].integrity_hash, "dummyhash");
}