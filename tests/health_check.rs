use std::future::IntoFuture;

use axum_newsletter::configuration::get_configuration;
use axum_newsletter::models::Subscriptions;
use diesel::prelude::*;
use diesel::SelectableHelper;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

pub struct TestApp {
    pub address: String,
    pub pool: Pool<AsyncPgConnection>,
}
async fn spawn_app() -> TestApp {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let configuration =
        get_configuration().expect("failed to get configuration");
    let pool_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(
        configuration.database.connection_string(),
    );
    let pool = Pool::builder(pool_manager)
        .build()
        .expect("Failed to create connection pool");

    let server = axum_newsletter::startup::run(listener, pool.clone());
    tokio::spawn(server.into_future());
    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        pool,
    }
}

#[tokio::test]
async fn health_check_responds_ok() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
#[tokio::test]
async fn subscribe_returns_200_for_valid_form() {
    use axum_newsletter::schema::subscriptions::dsl::*;
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let body = "name=Gabriel%20Aguiar&email=gabriel.masarin.aguiar%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
    let mut connection =
        test_app.pool.get().await.expect("Could not get connection");
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
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=Gabriel%20Aguiar", "missing email"),
        ("email=gabriel.masarin.aguiar%40gmail.com", "missing name"),
        ("", "missing both"),
    ];

    for (body, message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &test_app.address))
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
