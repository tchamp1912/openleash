use crate::tests::test_utils::setup_daemon;
use leash_ai_api::pb::request_service_server::RequestService;
use leash_ai_api::pb::ExecuteCommandRequest;
use tonic::Request;

#[tokio::test]
async fn test_execute_command_allowed() {
    let daemon = setup_daemon().await;
    
    let req = Request::new(ExecuteCommandRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        command: "echo".to_string(),
        args: vec!["hello".to_string()],
        reason: "test".to_string(),
        task_id: None,
        env_vars: std::collections::HashMap::new(),
        working_dir: None,
        timeout_seconds: 5,
    });

    let response = daemon.execute_command(req).await.unwrap();
    let res = response.into_inner();

    assert_eq!(res.status, "EXECUTED");
    assert_eq!(res.exit_code, 0);
    assert_eq!(res.stdout.trim(), "hello");
}

#[tokio::test]
async fn test_execute_command_timeout() {
    let daemon = setup_daemon().await;
    
    let req = Request::new(ExecuteCommandRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        command: "sleep".to_string(),
        args: vec!["10".to_string()],
        reason: "test timeout".to_string(),
        task_id: None,
        env_vars: std::collections::HashMap::new(),
        working_dir: None,
        timeout_seconds: 1, 
    });

    let response = daemon.execute_command(req).await.unwrap();
    let res = response.into_inner();

    assert_eq!(res.status, "TIMEOUT");
    assert_eq!(res.error_message, "Command timed out");
}
