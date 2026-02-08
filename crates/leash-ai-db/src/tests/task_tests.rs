use crate::db::Db;
use leash_ai_core::models::{Task, TaskStatus, Lease, LeaseStatus, PackageManager};
use uuid::Uuid;
use chrono::Utc;

#[tokio::test]
async fn test_task_lease_relationship() {
    let db = Db::new("sqlite::memory:").await.unwrap();
    let task_id = Uuid::new_v4();
    
    // 1. Create a task
    let task = Task {
        id: task_id,
        name: "test task".to_string(),
        scope_path: "/tmp/scope".to_string(),
        status: TaskStatus::Active,
        created_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::hours(1),
    };
    db.insert_task(&task).await.unwrap();

    // 2. Create a lease for that task
    let lease = Lease {
        id: Uuid::new_v4(),
        request_id: Uuid::new_v4(),
        task_id: Some(task_id),
        status: LeaseStatus::Active,
        manager: PackageManager::Pip,
        package_name: "tool".to_string(),
        package_version: "1.0".to_string(),
        scope_path: "/tmp/scope".to_string(),
        expires_at: Utc::now() + chrono::Duration::hours(1),
    };
    db.insert_lease(&lease).await.unwrap();

    // 3. Retrieve leases for task
    let leases = db.get_leases_by_task(&task_id).await.unwrap();
    assert_eq!(leases.len(), 1);
    assert_eq!(leases[0].package_name, "tool");
}
