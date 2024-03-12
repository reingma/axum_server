use crate::database::create_connection_pool;
use crate::{configuration::Settings, email_client::EmailClient, routes};
use axum::{extract::Request, routing, serve::Serve, Router};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection};
use secrecy::ExposeSecret;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info_span;
use uuid::Uuid;

pub fn run(
    listener: TcpListener,
    connection_pool: Pool<AsyncPgConnection>,
    email_client: Arc<EmailClient>,
) -> Serve<Router, Router> {
    let app: Router = Router::new()
        .route("/", routing::get(routes::greet))
        .route("/health_check", routing::get(routes::health_check))
        .route("/subscriptions", routing::post(routes::subscriptions))
        .layer(TraceLayer::new_for_http().make_span_with(
            |request: &Request<_>| {
                let request_id = Uuid::now_v7();
                info_span!("Http Request", %request_id, request_uri = %request.uri())
            },
        ))
        .with_state(connection_pool)
        .with_state(email_client);

    axum::serve(listener, app)
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
            server: run(listener, pool, Arc::new(email_client)),
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
