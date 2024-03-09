use axum_newsletter::configuration::get_configuration;
use axum_newsletter::configuration::DatabaseSettings;
use axum_newsletter::models::Subscriptions;
use axum_newsletter::telemetry::setup_tracing;
use diesel::prelude::*;
use diesel::SelectableHelper;
use diesel_async::async_connection_wrapper::AsyncConnectionWrapper;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::AsyncConnection;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel_migrations::embed_migrations;
use diesel_migrations::EmbeddedMigrations;
use diesel_migrations::MigrationHarness;
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use std::future::IntoFuture;
use std::sync::Arc;

const MIGRATION: EmbeddedMigrations = embed_migrations!();

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter = "debug";
    if std::env::var("TEST_LOG").is_ok() {
        setup_tracing("test", default_filter, std::io::stdout);
    } else {
        setup_tracing("test", default_filter, std::io::sink);
    }
});

pub struct TestApp {
    pub address: String,
    pub pool: Pool<AsyncPgConnection>,
}
async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let mut configuration =
        get_configuration().expect("failed to get configuration");
    configuration.database.database_name = uuid::Uuid::now_v7().to_string();

    configure_database(&configuration.database).await;
    let conn_string = configuration.database.connection_string().clone();
    tokio::task::spawn_blocking(move || {
        let mut db_conn: AsyncConnectionWrapper<AsyncPgConnection> =
            AsyncConnectionWrapper::<AsyncPgConnection>::establish(
                conn_string.expose_secret(),
            )
            .expect("Error");
        tokio::task::block_in_place(move || {
            db_conn.run_pending_migrations(MIGRATION).unwrap();
        })
    })
    .await
    .expect("thread panic");
    let pool = axum_newsletter::database::create_connection_pool(
        configuration.database.connection_string().expose_secret(),
    );

    let timeout = configuration.email_client.timeout();
    let email_client = axum_newsletter::email_client::EmailClient::new(
        &configuration.email_client.base_url,
        configuration
            .email_client
            .sender()
            .expect("No sender defined"),
        configuration.email_client.api_token,
        timeout,
    );

    let server = axum_newsletter::startup::run(
        listener,
        pool.clone(),
        Arc::new(email_client),
    );
    tokio::spawn(server.into_future());
    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        pool,
    }
}

async fn configure_database(db_settings: &DatabaseSettings) {
    let mut db_conn = AsyncPgConnection::establish(
        db_settings
            .connection_string_without_database()
            .expose_secret(),
    )
    .await
    .expect("Failed to connect");
    let query = diesel::sql_query(format!(
        r#"CREATE DATABASE "{}";"#,
        db_settings.database_name
    ));
    query
        .execute(&mut db_conn)
        .await
        .expect("Failed to create database");
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
#[tokio::test]
async fn subscribe_returns_400_when_fields_are_present_but_empty() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        (
            "name= &email=gabriel.masarin.aguiar%40gmail.com",
            "missing name",
        ),
        ("name=Gabriel%20Aguiar&email=", "missing email"),
        ("name=person&email=totallynotemail", "missing both"),
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
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            message
        );
    }
}
