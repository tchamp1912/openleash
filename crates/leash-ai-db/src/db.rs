use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use leash_ai_core::{models::{Lease, LeaseStatus, AuditEvent, ResourceType, PackageManager}, Result, LeashError};
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub struct Db {
    pool: Pool<Sqlite>,
}

impl Db {
    pub async fn new(url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await
            .map_err(|e| LeashError::Internal(e.to_string()))?;

        let db = Self { pool };
        db.init().await?;
        Ok(db)
    }

    async fn init(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                scope_path TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                expires_at DATETIME NOT NULL
            )"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| LeashError::Internal(e.to_string()))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS leases (
                id TEXT PRIMARY KEY,
                request_id TEXT NOT NULL,
                task_id TEXT,
                status TEXT NOT NULL,
                manager TEXT NOT NULL,
                package_name TEXT NOT NULL,
                package_version TEXT NOT NULL,
                scope_path TEXT NOT NULL,
                expires_at DATETIME NOT NULL
            )"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| LeashError::Internal(e.to_string()))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS audit_events (
                id TEXT PRIMARY KEY,
                timestamp DATETIME NOT NULL,
                event_type TEXT NOT NULL,
                actor TEXT NOT NULL,
                resource_type TEXT NOT NULL,
                resource_id TEXT NOT NULL,
                action TEXT NOT NULL,
                status TEXT NOT NULL,
                metadata TEXT NOT NULL,
                integrity_hash TEXT NOT NULL
            )"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| LeashError::Internal(e.to_string()))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS integrity (
                last_hash TEXT PRIMARY KEY
            )"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| LeashError::Internal(e.to_string()))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS pending_approvals (
                id TEXT PRIMARY KEY,
                request_id TEXT NOT NULL,
                task_id TEXT,
                resource_type TEXT NOT NULL,
                resource_id TEXT NOT NULL,
                reason TEXT NOT NULL,
                status TEXT NOT NULL,
                scope TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                expires_at DATETIME NOT NULL
            )"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| LeashError::Internal(e.to_string()))?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS approved_resources (
                id TEXT PRIMARY KEY,
                resource_type TEXT NOT NULL,
                resource_id TEXT NOT NULL,
                scope TEXT NOT NULL,
                task_id TEXT,
                expires_at DATETIME
            )"
        )
        .execute(&self.pool)
        .await
        .map_err(|e| LeashError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn insert_approval_request(&self, req: &leash_ai_core::models::ApprovalRequest) -> Result<()> {
        sqlx::query(
            "INSERT INTO pending_approvals (id, request_id, task_id, resource_type, resource_id, reason, status, scope, created_at, expires_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(req.id.to_string())
        .bind(req.request_id.to_string())
        .bind(req.task_id.map(|u| u.to_string()))
        .bind(format!("{:?}", req.resource_type))
        .bind(&req.resource_id)
        .bind(&req.reason)
        .bind(&req.status)
        .bind(format!("{:?}", req.scope))
        .bind(req.created_at)
        .bind(req.expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| LeashError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn get_pending_approvals(&self) -> Result<Vec<leash_ai_core::models::ApprovalRequest>> {
        let rows: Vec<(String, String, Option<String>, String, String, String, String, String, DateTime<Utc>, DateTime<Utc>)> = sqlx::query_as(
            "SELECT id, request_id, task_id, resource_type, resource_id, reason, status, scope, created_at, expires_at FROM pending_approvals WHERE status = 'Pending'"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| LeashError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|row| leash_ai_core::models::ApprovalRequest {
            id: Uuid::parse_str(&row.0).unwrap(),
            request_id: Uuid::parse_str(&row.1).unwrap(),
            task_id: row.2.map(|u| Uuid::parse_str(&u).unwrap()),
            resource_type: match row.3.to_lowercase().as_str() {
                "secret" => ResourceType::Secret,
                "package" => ResourceType::Package,
                "command" => ResourceType::Command,
                "system" => ResourceType::System,
                _ => ResourceType::Package,
            },
            resource_id: row.4,
            reason: row.5,
            status: row.6,
            scope: match row.7.as_str() {
                "Once" => leash_ai_core::models::ApprovalScope::Once,
                "Task" => leash_ai_core::models::ApprovalScope::Task,
                "Permanent" => leash_ai_core::models::ApprovalScope::Permanent,
                _ => leash_ai_core::models::ApprovalScope::Once,
            },
            created_at: row.8,
            expires_at: row.9,
        }).collect())
    }

    pub async fn get_approval_request(&self, id: &Uuid) -> Result<leash_ai_core::models::ApprovalRequest> {
        let row: (String, String, Option<String>, String, String, String, String, String, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            "SELECT id, request_id, task_id, resource_type, resource_id, reason, status, scope, created_at, expires_at FROM pending_approvals WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| LeashError::NotFound(format!("Approval request not found: {}", e)))?;

        Ok(leash_ai_core::models::ApprovalRequest {
            id: Uuid::parse_str(&row.0).unwrap(),
            request_id: Uuid::parse_str(&row.1).unwrap(),
            task_id: row.2.map(|u| Uuid::parse_str(&u).unwrap()),
            resource_type: match row.3.to_lowercase().as_str() {
                "secret" => ResourceType::Secret,
                "package" => ResourceType::Package,
                "command" => ResourceType::Command,
                "system" => ResourceType::System,
                _ => ResourceType::Package,
            },
            resource_id: row.4,
            reason: row.5,
            status: row.6,
            scope: match row.7.as_str() {
                "Once" => leash_ai_core::models::ApprovalScope::Once,
                "Task" => leash_ai_core::models::ApprovalScope::Task,
                "Permanent" => leash_ai_core::models::ApprovalScope::Permanent,
                _ => leash_ai_core::models::ApprovalScope::Once,
            },
            created_at: row.8,
            expires_at: row.9,
        })
    }

    pub async fn get_approval_by_id(&self, id: &Uuid) -> Result<Option<leash_ai_core::models::ApprovalRequest>> {
        match self.get_approval_request(id).await {
            Ok(req) => Ok(Some(req)),
            Err(LeashError::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn update_approval_status(&self, id: &Uuid, status: &str, override_scope: Option<leash_ai_core::models::ApprovalScope>) -> Result<()> {
        let mut tx = self.pool.begin().await
            .map_err(|e| LeashError::Internal(e.to_string()))?;

        sqlx::query("UPDATE pending_approvals SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| LeashError::Internal(e.to_string()))?;

        if status == "Approved" {
            // Fetch within transaction to avoid deadlock
            let row: (String, String, Option<String>, String, String, String, String, String, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
                "SELECT id, request_id, task_id, resource_type, resource_id, reason, status, scope, created_at, expires_at FROM pending_approvals WHERE id = ?"
            )
            .bind(id.to_string())
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| LeashError::NotFound(format!("Approval request not found during update: {}", e)))?;

            let scope = if let Some(s) = override_scope {
                s
            } else {
                match row.7.as_str() {
                    "Once" => leash_ai_core::models::ApprovalScope::Once,
                    "Task" => leash_ai_core::models::ApprovalScope::Task,
                    "Permanent" => leash_ai_core::models::ApprovalScope::Permanent,
                    _ => leash_ai_core::models::ApprovalScope::Once,
                }
            };

            if scope != leash_ai_core::models::ApprovalScope::Once {
                sqlx::query(
                    "INSERT INTO approved_resources (id, resource_type, resource_id, scope, task_id, expires_at)
                     VALUES (?, ?, ?, ?, ?, ?)"
                )
                .bind(Uuid::new_v4().to_string())
                .bind(row.3) // resource_type
                .bind(row.4) // resource_id
                .bind(format!("{:?}", scope))
                .bind(row.2) // task_id
                .bind(row.9) // expires_at
                .execute(&mut *tx)
                .await
                .map_err(|e| LeashError::Internal(e.to_string()))?;
            }
        }

        tx.commit().await
            .map_err(|e| LeashError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn check_approval(&self, resource_type: ResourceType, resource_id: &str, task_id: Option<Uuid>) -> Result<bool> {
        let mut query = "SELECT id FROM approved_resources 
             WHERE resource_type = ? AND resource_id = ? AND (expires_at IS NULL OR expires_at > ?)".to_string();
        
        if let Some(_) = task_id {
            query.push_str(" AND (scope = 'Permanent' OR (scope = 'Task' AND task_id = ?))");
        } else {
            query.push_str(" AND scope = 'Permanent'");
        }

        let mut q = sqlx::query_as::<_, (String,)>(&query)
            .bind(format!("{:?}", resource_type))
            .bind(resource_id)
            .bind(Utc::now());
        
        if let Some(tid) = task_id {
            q = q.bind(tid.to_string());
        }

        let row = q.fetch_optional(&self.pool)
            .await
            .map_err(|e| LeashError::Internal(e.to_string()))?;

        Ok(row.is_some())
    }

    pub async fn get_approval_by_request_id(&self, request_id: &Uuid) -> Result<Option<leash_ai_core::models::ApprovalRequest>> {
        let row: Option<(String, String, Option<String>, String, String, String, String, String, DateTime<Utc>, DateTime<Utc>)> = sqlx::query_as(
            "SELECT id, request_id, task_id, resource_type, resource_id, reason, status, scope, created_at, expires_at FROM pending_approvals WHERE request_id = ?"
        )
        .bind(request_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| LeashError::Internal(e.to_string()))?;

        if let Some(r) = row {
            Ok(Some(leash_ai_core::models::ApprovalRequest {
                id: Uuid::parse_str(&r.0).unwrap(),
                request_id: Uuid::parse_str(&r.1).unwrap(),
                task_id: r.2.map(|u| Uuid::parse_str(&u).unwrap()),
                resource_type: match r.3.to_lowercase().as_str() {
                    "secret" => ResourceType::Secret,
                    "package" => ResourceType::Package,
                    "command" => ResourceType::Command,
                    "system" => ResourceType::System,
                    _ => ResourceType::Package,
                },
                resource_id: r.4,
                reason: r.5,
                status: r.6,
                scope: match r.7.as_str() {
                    "Once" => leash_ai_core::models::ApprovalScope::Once,
                    "Task" => leash_ai_core::models::ApprovalScope::Task,
                    "Permanent" => leash_ai_core::models::ApprovalScope::Permanent,
                    _ => leash_ai_core::models::ApprovalScope::Once,
                },
                created_at: r.8,
                expires_at: r.9,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn insert_task(&self, task: &leash_ai_core::models::Task) -> Result<()> {
        sqlx::query(
            "INSERT INTO tasks (id, name, scope_path, status, created_at, expires_at)
             VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(task.id.to_string())
        .bind(&task.name)
        .bind(&task.scope_path)
        .bind(format!("{:?}", task.status))
        .bind(task.created_at)
        .bind(task.expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| LeashError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn get_task(&self, id: &Uuid) -> Result<leash_ai_core::models::Task> {
        let row: (String, String, String, String, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            "SELECT id, name, scope_path, status, created_at, expires_at FROM tasks WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| LeashError::NotFound(format!("Task not found: {}", e)))?;

        Ok(leash_ai_core::models::Task {
            id: Uuid::parse_str(&row.0).unwrap(),
            name: row.1,
            scope_path: row.2,
            status: match row.3.as_str() {
                "Active" => leash_ai_core::models::TaskStatus::Active,
                "Completed" => leash_ai_core::models::TaskStatus::Completed,
                "Expired" => leash_ai_core::models::TaskStatus::Expired,
                _ => leash_ai_core::models::TaskStatus::Active,
            },
            created_at: row.4,
            expires_at: row.5,
        })
    }

    pub async fn get_expired_tasks(&self) -> Result<Vec<leash_ai_core::models::Task>> {
        let rows: Vec<(String, String, String, String, DateTime<Utc>, DateTime<Utc>)> = sqlx::query_as(
            "SELECT id, name, scope_path, status, created_at, expires_at FROM tasks WHERE status = 'Active' AND expires_at < ?"
        )
        .bind(Utc::now())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| LeashError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|row| leash_ai_core::models::Task {
            id: Uuid::parse_str(&row.0).unwrap(),
            name: row.1,
            scope_path: row.2,
            status: leash_ai_core::models::TaskStatus::Active,
            created_at: row.4,
            expires_at: row.5,
        }).collect())
    }

    pub async fn update_task_status(&self, id: &Uuid, status: leash_ai_core::models::TaskStatus) -> Result<()> {
        sqlx::query("UPDATE tasks SET status = ? WHERE id = ?")
            .bind(format!("{:?}", status))
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| LeashError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_approvals_by_task(&self, task_id: &Uuid) -> Result<()> {
        sqlx::query("DELETE FROM approved_resources WHERE task_id = ?")
            .bind(task_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| LeashError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn get_leases_by_task(&self, task_id: &Uuid) -> Result<Vec<Lease>> {
        let rows: Vec<(String, String, Option<String>, String, String, String, String, String, DateTime<Utc>)> = sqlx::query_as(
            "SELECT id, request_id, task_id, status, manager, package_name, package_version, scope_path, expires_at FROM leases WHERE task_id = ?"
        )
        .bind(task_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| LeashError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|row| Lease {
            id: Uuid::parse_str(&row.0).unwrap(),
            request_id: Uuid::parse_str(&row.1).unwrap(),
            task_id: row.2.map(|u| Uuid::parse_str(&u).unwrap()),
            status: match row.3.as_str() {
                "Active" => LeaseStatus::Active,
                "Expired" => LeaseStatus::Expired,
                "Revoked" => LeaseStatus::Revoked,
                _ => LeaseStatus::Active,
            },
            manager: match row.4.as_str() {
                "Pip" => PackageManager::Pip,
                "Npm" => PackageManager::Npm,
                "Brew" => PackageManager::Brew,
                _ => PackageManager::Pip,
            },
            package_name: row.5,
            package_version: row.6,
            scope_path: row.7,
            expires_at: row.8,
        }).collect())
    }

    pub async fn update_lease_status(&self, id: &Uuid, status: LeaseStatus) -> Result<()> {
        sqlx::query("UPDATE leases SET status = ? WHERE id = ?")
            .bind(format!("{:?}", status))
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| LeashError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn insert_lease(&self, lease: &Lease) -> Result<()> {
        sqlx::query(
            "INSERT INTO leases (id, request_id, task_id, status, manager, package_name, package_version, scope_path, expires_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(lease.id.to_string())
        .bind(lease.request_id.to_string())
        .bind(lease.task_id.map(|u| u.to_string()))
        .bind(format!("{:?}", lease.status))
        .bind(format!("{:?}", lease.manager))
        .bind(&lease.package_name)
        .bind(&lease.package_version)
        .bind(&lease.scope_path)
        .bind(lease.expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| LeashError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn get_expired_leases(&self) -> Result<Vec<Lease>> {
        let rows: Vec<(String, String, Option<String>, String, String, String, String, String, DateTime<Utc>)> = sqlx::query_as(
            "SELECT id, request_id, task_id, status, manager, package_name, package_version, scope_path, expires_at FROM leases WHERE status = 'Active' AND task_id IS NULL AND expires_at < ?"
        )
        .bind(Utc::now())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| LeashError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|row| Lease {
            id: Uuid::parse_str(&row.0).unwrap(),
            request_id: Uuid::parse_str(&row.1).unwrap(),
            task_id: row.2.map(|u| Uuid::parse_str(&u).unwrap()),
            status: LeaseStatus::Active,
            manager: match row.4.as_str() {
                "Pip" => PackageManager::Pip,
                "Npm" => PackageManager::Npm,
                "Brew" => PackageManager::Brew,
                _ => PackageManager::Pip,
            },
            package_name: row.5,
            package_version: row.6,
            scope_path: row.7,
            expires_at: row.8,
        }).collect())
    }

    pub async fn get_last_integrity_hash(&self) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT last_hash FROM integrity LIMIT 1")
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| LeashError::Internal(e.to_string()))?;
        Ok(row.map(|r| r.0))
    }

    pub async fn insert_audit_event(&self, event: &AuditEvent) -> Result<()> {
        let mut tx = self.pool.begin().await
            .map_err(|e| LeashError::Internal(e.to_string()))?;

        sqlx::query(
            "INSERT INTO audit_events (id, timestamp, event_type, actor, resource_type, resource_id, action, status, metadata, integrity_hash)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(event.id.to_string())
        .bind(event.timestamp)
        .bind(&event.event_type)
        .bind(&event.actor)
        .bind(format!("{:?}", event.resource_type))
        .bind(&event.resource_id)
        .bind(&event.action)
        .bind(&event.status)
        .bind(serde_json::to_string(&event.metadata).unwrap_or_default())
        .bind(&event.integrity_hash)
        .execute(&mut *tx)
        .await
        .map_err(|e| LeashError::Internal(e.to_string()))?;

        sqlx::query("DELETE FROM integrity").execute(&mut *tx).await
            .map_err(|e| LeashError::Internal(e.to_string()))?;
        sqlx::query("INSERT INTO integrity (last_hash) VALUES (?)")
            .bind(&event.integrity_hash)
            .execute(&mut *tx)
            .await
            .map_err(|e| LeashError::Internal(e.to_string()))?;

        tx.commit().await
            .map_err(|e| LeashError::Internal(e.to_string()))?;

        Ok(())
    }

    pub async fn query_audit_logs(&self, start_time: Option<DateTime<Utc>>, end_time: Option<DateTime<Utc>>, limit: u32) -> Result<Vec<AuditEvent>> {
        let mut query = "SELECT id, timestamp, event_type, actor, resource_type, resource_id, action, status, metadata, integrity_hash FROM audit_events".to_string();
        let mut conditions = Vec::new();
        
        if start_time.is_some() {
            conditions.push("timestamp >= ?".to_string());
        }
        if end_time.is_some() {
            conditions.push("timestamp <= ?".to_string());
        }
        
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
        
        query.push_str(" ORDER BY timestamp DESC LIMIT ?");
        
        let mut sql_query = sqlx::query_as::<_, (String, DateTime<Utc>, String, String, String, String, String, String, String, String)>(&query);
        
        if let Some(st) = start_time {
            sql_query = sql_query.bind(st);
        }
        if let Some(et) = end_time {
            sql_query = sql_query.bind(et);
        }
        sql_query = sql_query.bind(limit);
        
        let rows = sql_query.fetch_all(&self.pool).await
            .map_err(|e| LeashError::Internal(e.to_string()))?;

        Ok(rows.into_iter().map(|row| AuditEvent {
            id: Uuid::parse_str(&row.0).unwrap(),
            timestamp: row.1,
            event_type: row.2,
            actor: row.3,
            resource_type: match row.4.to_lowercase().as_str() {
                "secret" => ResourceType::Secret,
                "package" => ResourceType::Package,
                "command" => ResourceType::Command,
                "system" => ResourceType::System,
                _ => ResourceType::Package,
            },
            resource_id: row.5,
            action: row.6,
            status: row.7,
            metadata: serde_json::from_str(&row.8).unwrap_or_default(),
            integrity_hash: row.9,
        }).collect())
    }
}