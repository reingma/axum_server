use crate::database::{queries::insert_subscriber, DatabaseConnectionPool};
use axum::{extract::State, http::StatusCode, Form};
use unicode_segmentation::UnicodeSegmentation;

#[derive(serde::Deserialize)]
pub struct Subscriber {
    pub name: String,
    pub email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(subscriber,pool),
    fields(
        subscriber_email = %subscriber.email,
        subscriber_name = %subscriber.name
    )
)]
pub async fn subscriptions(
    State(pool): State<DatabaseConnectionPool>,
    Form(subscriber): Form<Subscriber>,
) -> StatusCode {
    if !is_valid_name(&subscriber.name) {
        return StatusCode::BAD_REQUEST;
    }
    tracing::info!("Adding a new subscriber to the database.");

    let mut connection = crate::database::get_connection(pool).await;

    match insert_subscriber(subscriber, &mut connection).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn is_valid_name(s: &str) -> bool {
    let is_empty_or_whitespace = s.trim().is_empty();
    let is_too_long = s.graphemes(true).count() > 256;
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let has_forbidden_chars =
        s.chars().any(|g| forbidden_characters.contains(&g));
    !is_too_long && !is_empty_or_whitespace && !has_forbidden_chars
}
