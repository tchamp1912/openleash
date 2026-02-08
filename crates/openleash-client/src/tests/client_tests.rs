use crate::client::OpenLeashClient;

#[tokio::test]
async fn test_client_connect_invalid_uri() {
    // Should fail because nothing is listening
    let result = OpenLeashClient::connect("http://127.0.0.1:12345".to_string()).await;
    assert!(result.is_err());
}
