use crate::{
    database::{queries::insert_subscriber, DatabaseConnectionPool},
    domain::NewSubscriber,
};
use axum::{extract::State, http::StatusCode, Form};

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
    let new_subscriber: NewSubscriber = match subscriber.try_into() {
        Ok(new_subscriber) => new_subscriber,
        Err(_) => return StatusCode::BAD_REQUEST,
    };
    tracing::info!("Adding a new subscriber to the database.");

    let mut connection = crate::database::get_connection(pool).await;

    match insert_subscriber(new_subscriber, &mut connection).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
