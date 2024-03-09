use std::sync::Arc;

use crate::{email_client::EmailClient, routes};
use axum::{extract::Request, routing, serve::Serve, Router};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection};
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
