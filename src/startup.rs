use crate::routes;
use axum::{routing, serve::Serve, Router};
use tokio::net::TcpListener;

pub fn run(listener: TcpListener) -> Serve<Router, Router> {
    let app: Router = Router::new()
        .route("/", routing::get(routes::greet))
        .route("/health_check", routing::get(routes::health_check))
        .route("/subscriptions", routing::post(routes::subscriptions));

    axum::serve(listener, app)
}
