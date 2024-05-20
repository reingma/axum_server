use crate::database::{create_connection_pool, DatabaseConnectionPool};
use crate::{configuration::Settings, email_client::EmailClient, routes};
use axum::response::Response;
use axum::{extract::Request, routing, serve::Serve, Router};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection};
use secrecy::ExposeSecret;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info_span, Span};
use uuid::Uuid;

pub fn run(
    listener: TcpListener,
    connection_pool: Pool<AsyncPgConnection>,
    email_client: Arc<EmailClient>,
    base_url: String,
) -> Serve<Router, Router> {
    let app_state = ApplicationState {
        database_pool: connection_pool,
        email_client,
        base_url,
    };
    let app: Router = Router::new()
        .route("/", routing::get(routes::greet))
        .route("/health_check", routing::get(routes::health_check))
        .route("/subscriptions", routing::post(routes::subscriptions))
        .route("/subscriptions/confirm", routing::get(routes::confirm))
        .layer(TraceLayer::new_for_http().make_span_with(
            |request: &Request<_>| {
                let request_id = Uuid::now_v7();
                info_span!("Http Request", %request_id, request_uri = %request.uri(), response_code = tracing::field::Empty)
            }
        ).on_response(|response: &Response, _latency: Duration, span: &Span|{
                span.record("response_code", response.status().as_str());
            }))
        .with_state(app_state);

    axum::serve(listener, app)
}

#[derive(Clone)]
pub struct ApplicationState {
    pub database_pool: DatabaseConnectionPool,
    pub email_client: Arc<EmailClient>,
    pub base_url: String,
}

pub struct Application {
    port: u16,
    pool: Pool<AsyncPgConnection>,
    server: Serve<Router, Router>,
}

impl Application {
    pub async fn build(
        configuration: Settings,
    ) -> Result<Application, std::io::Error> {
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

        Ok(Application {
            pool: pool.clone(),
            server: run(
                listener,
                pool,
                Arc::new(email_client),
                configuration.application.base_url,
            ),
            port,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn pool(&self) -> Pool<AsyncPgConnection> {
        self.pool.clone()
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
