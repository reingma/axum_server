use axum_newsletter::configuration::get_configuration;
use axum_newsletter::configuration::DatabaseSettings;
use axum_newsletter::database::DatabaseConnection;
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
use reqwest::Client;
use secrecy::ExposeSecret;
use std::future::IntoFuture;

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
impl TestApp {
    pub async fn subscribe(
        &self,
        body: String,
    ) -> Result<reqwest::Response, reqwest::Error> {
        Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
    }

    pub async fn check_health(
        &self,
    ) -> Result<reqwest::Response, reqwest::Error> {
        Client::new()
            .get(&format!("{}/health_check", &self.address))
            .send()
            .await
    }
}
pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("failed to get configuration");
        c.database.database_name = uuid::Uuid::now_v7().to_string();
        c.application.port = 0;
        c
    };

    configure_database(&configuration.database).await;

    let application =
        axum_newsletter::startup::Application::build(configuration)
            .await
            .expect("Failed to build app.");
    let testapp = TestApp {
        address: format!("http://127.0.0.1:{}", application.port()),
        pool: application.pool(),
    };
    tokio::spawn(application.run_until_stopped().into_future());
    testapp
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
    let conn_string = db_settings.connection_string().clone();
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
}

pub async fn check_subscriber_existance(
    connection: &mut DatabaseConnection,
    subscriber_email: &str,
) -> Vec<axum_newsletter::models::Subscriptions> {
    use axum_newsletter::schema::subscriptions::dsl::*;
    subscriptions
        .limit(1)
        .filter(email.eq(subscriber_email))
        .select(Subscriptions::as_select())
        .load(connection)
        .await
        .expect("Failed to read query")
}
