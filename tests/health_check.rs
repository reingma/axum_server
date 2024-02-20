use std::future::IntoFuture;

use axum_newsletter::{configuration, models::Subscriptions};
use diesel::prelude::*;
use diesel::SelectableHelper;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};

async fn spawn_app() -> String {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = axum_newsletter::startup::run(listener);
    tokio::spawn(server.into_future());
    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_responds_ok() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
#[tokio::test]
async fn subscribe_returns_200_for_valid_form() {
    use axum_newsletter::schema::subscriptions::dsl::*;
    let address = spawn_app().await;

    let client = reqwest::Client::new();

    let body = "name=Gabriel%20Aguiar&email=gabriel.masarin.aguiar%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
    let configuration = configuration::get_configuration()
        .expect("Failed to read configuration");
    let connection_str = configuration.database.connection_string();
    let mut connection = AsyncPgConnection::establish(&connection_str)
        .await
        .expect("Failed to connect to postgres.");
    let results = subscriptions
        .limit(1)
        .filter(name.eq("Gabriel Aguiar"))
        .select(Subscriptions::as_select())
        .load(&mut connection)
        .await
        .expect("Failed to read query");
    assert_eq!(results.len(), 1);
}
#[tokio::test]
async fn subscribe_returns_422_when_data_is_missing() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=Gabriel%20Aguiar", "missing email"),
        ("email=gabriel.masarin.aguiar%40gmail.com", "missing name"),
        ("", "missing both"),
    ];

    for (body, message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 422 Bad Request when the payload was {}",
            message
        );
    }
}
