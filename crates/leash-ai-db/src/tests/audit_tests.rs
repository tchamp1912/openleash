use crate::db::Db;
use leash_ai_core::models::{AuditEvent, ResourceType};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

#[tokio::test]
async fn test_audit_integrity_chain() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    
    // 1. Insert first event
    let mut event1 = AuditEvent {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        event_type: "TEST".to_string(),
        actor: "me".to_string(),
        resource_type: ResourceType::Package,
        resource_id: "pkg1".to_string(),
        action: "INSTALL".to_string(),
        status: "SUCCESS".to_string(),
        metadata: HashMap::new(),
        integrity_hash: String::new(),
    };
    event1.integrity_hash = event1.calculate_integrity_hash("START");
    db.insert_audit_event(&event1).await.unwrap();

    // 2. Insert second event
    let mut event2 = AuditEvent {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        event_type: "TEST".to_string(),
        actor: "me".to_string(),
        resource_type: ResourceType::Package,
        resource_id: "pkg2".to_string(),
        action: "INSTALL".to_string(),
        status: "SUCCESS".to_string(),
        metadata: HashMap::new(),
        integrity_hash: String::new(),
    };
    let last_hash = db.get_last_integrity_hash().await.unwrap().unwrap();
    assert_eq!(last_hash, event1.integrity_hash);
    
    event2.integrity_hash = event2.calculate_integrity_hash(&last_hash);
    db.insert_audit_event(&event2).await.unwrap();

    // 3. Verify final hash
    let final_hash = db.get_last_integrity_hash().await.unwrap().unwrap();
    assert_eq!(final_hash, event2.integrity_hash);
}
