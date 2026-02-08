use tonic::{Request, Response, Status};
use openleash_api::pb::task_service_server::TaskService;
use openleash_api::pb::{StartTaskRequest, StartTaskResponse, EndTaskRequest, EndTaskResponse, GetTaskEnvironmentRequest, GetTaskEnvironmentResponse};
use openleash_core::models::{Task, TaskStatus, ResourceType};
use openleash_venv::VenvManager;
use crate::OpenLeashDaemon;
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

#[tonic::async_trait]
impl TaskService for OpenLeashDaemon {
    async fn start_task(
        &self,
        request: Request<StartTaskRequest>,
    ) -> std::result::Result<Response<StartTaskResponse>, Status> {
        let req = request.into_inner();
        let ttl_seconds = req.ttl_seconds.clamp(60, 86400); 

        let task_id = Uuid::new_v4();
        let scope_path = format!("{}/task-{}", req.base_scope_path, task_id);
        
        let task = Task {
            id: task_id,
            name: req.name.clone(),
            scope_path: scope_path.clone(),
            status: TaskStatus::Active,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::seconds(ttl_seconds as i64),
        };

        let venv = VenvManager::new(&scope_path);
        venv.ensure_created().await
            .map_err(|e| Status::internal(format!("Failed to create task venv: {}", e)))?;

        self.db.insert_task(&task)
            .await
            .map_err(|e| Status::internal(format!("Failed to store task: {}", e)))?;

        self.audit("TASK", "agent", ResourceType::System, &task_id.to_string(), "START", "SUCCESS", HashMap::from([("name".to_string(), req.name)])).await?;

        tracing::info!(task_id = %task_id, name = %task.name, "Task started");

        Ok(Response::new(StartTaskResponse {
            task_id: task_id.to_string(),
            scope_path,
        }))
    }

    async fn end_task(
        &self,
        request: Request<EndTaskRequest>,
    ) -> std::result::Result<Response<EndTaskResponse>, Status> {
        let req = request.into_inner();
        let task_id = Uuid::parse_str(&req.task_id)
            .map_err(|_| Status::invalid_argument("Invalid task_id"))?;

        self.cleanup_task(&task_id, TaskStatus::Completed).await?;
        
        self.audit("TASK", "agent", ResourceType::System, &req.task_id, "END", "SUCCESS", HashMap::new()).await?;

        tracing::info!(task_id = %task_id, "Task ended");

        Ok(Response::new(EndTaskResponse { success: true }))
    }

    async fn get_task_environment(
        &self,
        request: Request<GetTaskEnvironmentRequest>,
    ) -> std::result::Result<Response<GetTaskEnvironmentResponse>, Status> {
        let req = request.into_inner();
        let task_id = Uuid::parse_str(&req.task_id)
            .map_err(|_| Status::invalid_argument("Invalid task_id"))?;

        let task = self.db.get_task(&task_id)
            .await
            .map_err(|_| Status::failed_precondition("Task not found or inactive"))?;
        
        if task.status != TaskStatus::Active {
            return Err(Status::failed_precondition("Task is no longer active"));
        }

        let venv = VenvManager::new(&task.scope_path);
        let bin_path = venv.bin_dir().to_string_lossy().to_string();

        Ok(Response::new(GetTaskEnvironmentResponse {
            bin_path,
            scope_path: task.scope_path,
            error_message: String::new(),
        }))
    }
}
