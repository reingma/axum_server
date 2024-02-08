use axum::{http::StatusCode, response::Html, routing, serve::Serve, Router};
use tokio::net::TcpListener;

pub fn run(listener: TcpListener) -> Serve<Router, Router> {
    let app: Router = Router::new()
        .route("/", routing::get(greet))
        .route("/health_check", routing::get(health_check))
        .route("/subscriptions", routing::post(subscriptions));

    axum::serve(listener, app)
}

async fn greet() -> Html<&'static str> {
    Html("<h1>Hello</h1>")
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn subscriptions() -> StatusCode {
    StatusCode::OK
}
