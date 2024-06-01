use argon2::password_hash::SaltString;
use argon2::Algorithm;
use argon2::Argon2;
use argon2::Params;
use argon2::PasswordHasher;
use argon2::Version;
use axum_newsletter::configuration::get_configuration;
use axum_newsletter::configuration::DatabaseSettings;
use axum_newsletter::database::DatabaseConnection;
use axum_newsletter::models::Subscriptions;
use axum_newsletter::models::Users;
use axum_newsletter::schema::users;
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
use rand::distributions::Alphanumeric;
use rand::thread_rng;
use rand::Rng;
use reqwest::Client;
use secrecy::ExposeSecret;
use std::future::IntoFuture;
use uuid::Uuid;
use wiremock::MockServer;

const MIGRATION: EmbeddedMigrations = embed_migrations!();

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter = "debug";
    if std::env::var("TEST_LOG").is_ok() {
        setup_tracing("test", default_filter, std::io::stdout);
    } else {
        setup_tracing("test", default_filter, std::io::sink);
    }
});

pub struct TestUser {
    pub user_id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::now_v7(),
            username: Uuid::now_v7().to_string(),
            password: Uuid::now_v7().to_string(),
        }
    }
    pub async fn store(&self, connection: &mut DatabaseConnection) {
        let salt = SaltString::generate(&mut rand::thread_rng());
        let hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        )
        .hash_password(self.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

        let user = Users::new(self.user_id, &self.username, &hash);
        diesel::insert_into(users::table)
            .values(&user)
            .execute(connection)
            .await
            .expect("Failed to add user");
    }

    pub fn get_credentials(&self) -> (&str, &str) {
        (&self.username, &self.password)
    }
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub struct TestApp {
    pub address: String,
    pub pool: Pool<AsyncPgConnection>,
    pub email_server: MockServer,
    pub server_port: u16,
    pub test_user: TestUser,
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

    pub async fn get_confirmation_links(
        &self,
        email_request: &wiremock::Request,
    ) -> ConfirmationLinks {
        let body: serde_json::Value =
            serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.server_port)).unwrap();
            confirmation_link
        };
        let html = get_link(body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(body["TextBody"].as_str().unwrap());
        ConfirmationLinks { html, plain_text }
    }

    pub async fn post_newsletter(
        &self,
        body: serde_json::Value,
    ) -> reqwest::Response {
        let (uname, pword) = self.test_user.get_credentials();
        reqwest::Client::new()
            .post(&format!("{}/newsletters", &self.address))
            .basic_auth(uname, Some(pword))
            .json(&body)
            .send()
            .await
            .expect("Request failed.")
    }
}
pub async fn spawn_app(migration: Option<EmbeddedMigrations>) -> TestApp {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;
    let configuration = {
        let mut c = get_configuration().expect("failed to get configuration");
        c.database.database_name = uuid::Uuid::now_v7().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };

    configure_database(&configuration.database, migration).await;

    let application =
        axum_newsletter::startup::Application::build(configuration)
            .await
            .expect("Failed to build app.");
    let testapp = TestApp {
        address: format!("http://127.0.0.1:{}", application.port()),
        pool: application.pool(),
        email_server,
        server_port: application.port(),
        test_user: TestUser::generate(),
    };
    let mut connection = testapp
        .pool
        .get()
        .await
        .expect("Could not retrieve database connection");
    testapp.test_user.store(&mut connection).await;
    tokio::spawn(application.run_until_stopped().into_future());
    testapp
}

async fn configure_database(
    db_settings: &DatabaseSettings,
    migration: Option<EmbeddedMigrations>,
) {
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
    axum_newsletter::telemetry::spawn_blocking_with_tracing(move || {
        let mut db_conn: AsyncConnectionWrapper<AsyncPgConnection> =
            AsyncConnectionWrapper::<AsyncPgConnection>::establish(
                conn_string.expose_secret(),
            )
            .expect("Error");
        tokio::task::block_in_place(move || match migration {
            None => {
                db_conn.run_pending_migrations(MIGRATION).unwrap();
            }
            Some(test_migration) => {
                db_conn.run_pending_migrations(test_migration).unwrap();
            }
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

pub fn generate_valid_subscriber_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
