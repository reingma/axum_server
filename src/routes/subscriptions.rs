use axum::{http::StatusCode, Form};

#[derive(serde::Deserialize)]
pub struct Subscriber {
    name: String,
    email: String,
}

pub async fn subscriptions(Form(subscriber): Form<Subscriber>) -> StatusCode {
    StatusCode::OK
}
