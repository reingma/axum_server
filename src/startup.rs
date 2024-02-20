use crate::routes;
use axum::{routing, serve::Serve, Router};
use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection};
use tokio::net::TcpListener;

pub fn run(
    listener: TcpListener,
    connection_pool: Pool<AsyncPgConnection>,
) -> Serve<Router, Router> {
    let app: Router = Router::new()
        .route("/", routing::get(routes::greet))
        .route("/health_check", routing::get(routes::health_check))
        .route("/subscriptions", routing::post(routes::subscriptions))
        .with_state(connection_pool);

    axum::serve(listener, app)
}
