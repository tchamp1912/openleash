use crate::db::Db;
use openleash_core::models::{ApprovalRequest, ResourceType, ApprovalScope};
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_approval_persistence() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    let request_id = Uuid::new_v4();
    let approval_id = Uuid::new_v4();

    let req = ApprovalRequest {
        id: approval_id,
        request_id,
        task_id: None,
        resource_type: ResourceType::Secret,
        resource_id: "key".to_string(),
        reason: "need it".to_string(),
        status: "Pending".to_string(),
        scope: ApprovalScope::Once,
        created_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::minutes(15),
    };

    db.insert_approval_request(&req).await.unwrap();

    // Find by request_id
    let found = db.get_approval_by_request_id(&request_id).await.unwrap().unwrap();
    assert_eq!(found.id, approval_id);

    // Update status
    db.update_approval_status(&approval_id, "Approved", None).await.unwrap();
    let updated = db.get_pending_approvals().await.unwrap();
    assert!(updated.is_empty());
}

#[tokio::test]
async fn test_persistent_approval_check() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    let request_id = Uuid::new_v4();
    let approval_id = Uuid::new_v4();

    let req = ApprovalRequest {
        id: approval_id,
        request_id,
        task_id: None,
        resource_type: ResourceType::Package,
        resource_id: "numpy".to_string(),
        reason: "need it".to_string(),
        status: "Pending".to_string(),
        scope: ApprovalScope::Permanent,
        created_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::days(365),
    };

    db.insert_approval_request(&req).await.unwrap();
    
    // Not approved yet
    assert_eq!(db.check_approval(ResourceType::Package, "numpy", None).await.unwrap(), false);

    // Approve
    db.update_approval_status(&approval_id, "Approved", None).await.unwrap();

    // Now it should be approved persistently
    assert_eq!(db.check_approval(ResourceType::Package, "numpy", None).await.unwrap(), true);
}