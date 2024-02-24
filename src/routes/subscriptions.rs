use crate::DatabaseConnection;
use crate::{models::Subscriptions, schema::subscriptions};
use axum::{extract::State, http::StatusCode, Form};
use diesel_async::{
    pooled_connection::deadpool::Pool, AsyncPgConnection, RunQueryDsl,
};
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Subscriber {
    name: String,
    email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(subscriber,pool),
    fields(
        request_id = %Uuid::now_v7(),
        subscriber_email = %subscriber.email,
        subscriber_name = %subscriber.name
    )
)]
pub async fn subscriptions(
    State(pool): State<Pool<AsyncPgConnection>>,
    Form(subscriber): Form<Subscriber>,
) -> StatusCode {
    tracing::info!("Adding a new subscriber to the database.");

    let mut connection = get_connection(pool).await;

    insert_subscriber(subscriber, &mut connection).await
}

#[tracing::instrument(
    name = "Retrieving database connection from pool.",
    skip(pool)
)]
pub async fn get_connection(
    pool: Pool<AsyncPgConnection>,
) -> DatabaseConnection {
    let pooling_span =
        tracing::info_span!("Getting connection from database pool");
    match pool.get().instrument(pooling_span).await {
        Ok(conn) => {
            tracing::info!("Connection established.");
            conn
        }
        Err(e) => {
            tracing::error!(
                "Could not get connection from pool, with error: {:?}",
                e
            );
            panic!("Failed to establish connection.");
        }
    }
}

#[tracing::instrument(
    name = "Inserting subscriber into databse.",
    skip(subscriber_data, connection)
)]
pub async fn insert_subscriber(
    subscriber_data: Subscriber,
    connection: &mut DatabaseConnection,
) -> StatusCode {
    let subscription_entry =
        Subscriptions::new(subscriber_data.email, subscriber_data.name);
    match diesel::insert_into(subscriptions::table)
        .values(subscription_entry)
        .execute(connection)
        .await
    {
        Ok(_) => {
            tracing::info!("New subscriber details have been saved");
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!("Failed to execute query {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
