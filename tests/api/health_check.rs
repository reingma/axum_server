use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_responds_ok() {
    let test_app = spawn_app().await;
    let response = test_app
        .check_health()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
