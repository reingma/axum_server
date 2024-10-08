use crate::authentication::check_credentials;
use crate::database::{create_connection_pool, DatabaseConnectionPool};
use crate::{configuration::Settings, email_client::EmailClient, routes};
use axum::extract::FromRef;
use axum::middleware;
use axum::response::Response;
use axum::{extract::Request, routing, serve::Serve, Router};
use axum_extra::extract::cookie::Key;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection};
use secrecy::ExposeSecret;
use secrecy::Secret;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_sessions::SessionManagerLayer;
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};
use tracing::{info_span, Span};
use uuid::Uuid;

pub async fn run(
    listener: TcpListener,
    connection_pool: Pool<AsyncPgConnection>,
    email_client: Arc<EmailClient>,
    base_url: String,
    key: Key,
    redis_uri: Secret<String>,
) -> Result<(Serve<Router, Router>, RedisConnection), anyhow::Error> {
    let app_state = ApplicationState {
        database_pool: connection_pool,
        email_client,
        base_url,
        key,
    };
    let redis_pool = RedisPool::new(
        RedisConfig::from_url(redis_uri.expose_secret())?,
        None,
        None,
        None,
        6,
    )?;
    let redis_connection = redis_pool.connect();
    redis_pool.wait_for_connect().await?;
    let session_store = RedisStore::new(redis_pool);
    let session_layer = SessionManagerLayer::new(session_store);
    let tracing_layer = TraceLayer::new_for_http().make_span_with(
            |request: &Request<_>| {
                let request_id = Uuid::now_v7();
                info_span!("Http Request", %request_id, request_uri = %request.uri(), response_code = tracing::field::Empty)
            }
        ).on_response(|response: &Response, _latency: Duration, span: &Span|{
                span.record("response_code", response.status().as_str());
            });
    let admin_routes = Router::new()
        .route("/admin/dashboard", routing::get(routes::admin_dashboard))
        .route("/admin/password", routing::get(routes::reset_password_form))
        .route("/admin/password", routing::post(routes::change_pasword))
        .route("/admin/logout", routing::post(routes::logout))
        .route(
            "/admin/newsletters",
            routing::post(routes::publish_newsletter),
        )
        .route("/admin/newsletters", routing::get(routes::newsletters_form))
        .layer(
            ServiceBuilder::new()
                .layer(session_layer.clone())
                .layer(middleware::from_fn(check_credentials)),
        )
        .layer(tracing_layer.clone())
        .with_state(app_state.clone());

    let basic_routes: Router = Router::new()
        .route("/", routing::get(routes::home))
        .route("/health_check", routing::get(routes::health_check))
        .route("/subscriptions", routing::post(routes::subscriptions))
        .route("/subscriptions/confirm", routing::get(routes::confirm))
        .route("/login", routing::get(routes::login_form))
        .route("/login", routing::post(routes::login))
        .layer(session_layer)
        .layer(tracing_layer)
        .with_state(app_state);

    let app = basic_routes.merge(admin_routes);

    //    redis_connection.await??;
    Ok((axum::serve(listener, app), redis_connection))
}

#[derive(Clone)]
pub struct ApplicationState {
    pub database_pool: DatabaseConnectionPool,
    pub email_client: Arc<EmailClient>,
    pub base_url: String,
    key: Key,
}
impl FromRef<ApplicationState> for Key {
    fn from_ref(state: &ApplicationState) -> Self {
        state.key.clone()
    }
}

type RedisConnection = JoinHandle<Result<(), RedisError>>;
pub struct Application {
    port: u16,
    pool: Pool<AsyncPgConnection>,
    server: Serve<Router, Router>,
    redis_connection_handle: RedisConnection,
}

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);

impl Application {
    pub async fn build(
        configuration: Settings,
    ) -> Result<Application, anyhow::Error> {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            &configuration.email_client.base_url,
            configuration
                .email_client
                .sender()
                .expect("Sender email invalid"),
            configuration.email_client.api_token,
            timeout,
        );

        let listener = tokio::net::TcpListener::bind(address).await?;
        tracing::info!(
            "Server started listening on port {}",
            listener.local_addr().unwrap()
        );
        let pool = create_connection_pool(
            configuration.database.connection_string().expose_secret(),
        );
        let port = listener.local_addr().unwrap().port();
        let key = Key::from(
            configuration
                .application
                .hmac_secret
                .expose_secret()
                .as_bytes(),
        );
        let pool_clone = pool.clone();
        let (server, redis_connection_handle) = run(
            listener,
            pool,
            Arc::new(email_client),
            configuration.application.base_url,
            key,
            configuration.application.redis_uri,
        )
        .await?;

        Ok(Application {
            pool: pool_clone,
            server,
            port,
            redis_connection_handle,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn pool(&self) -> Pool<AsyncPgConnection> {
        self.pool.clone()
    }

    pub async fn run_until_stopped(self) -> Result<(), anyhow::Error> {
        self.server.await?;
        self.redis_connection_handle.await??;
        Ok(())
    }
}
