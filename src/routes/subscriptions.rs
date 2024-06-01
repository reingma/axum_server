use std::sync::Arc;

use crate::domain::SubscriptionToken;
use crate::{
    database::queries, database::queries::insert_subscriber,
    domain::NewSubscriber, email_client::send::send_confirmation_email,
    startup::ApplicationState,
};
use anyhow::Context;
use axum::response::IntoResponse;
use axum::{extract::State, http::StatusCode, Form};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::AsyncConnection;
use serde::Serialize;

#[derive(serde::Deserialize)]
pub struct Subscriber {
    pub name: String,
    pub email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(subscriber,app_state),
    fields(
        subscriber_email = %subscriber.email,
        subscriber_name = %subscriber.name
    )
)]
pub async fn subscriptions(
    State(app_state): State<ApplicationState>,
    Form(subscriber): Form<Subscriber>,
) -> Result<StatusCode, SubscriptionError> {
    let new_subscriber: NewSubscriber = subscriber
        .try_into()
        .map_err(SubscriptionError::InvalidSubscriberData)?;
    tracing::info!("Adding a new subscriber to the database.");

    let mut connection =
        crate::database::get_connection(app_state.database_pool)
            .await
            .context("Could not get connection from pool.")?;

    let subscription_token = Arc::new(SubscriptionToken::generate());
    connection
        .transaction::<_, SubscriptionError, _>(|conn| {
            async move {
                let subscriber_id = insert_subscriber(conn, &new_subscriber)
                    .await
                    .context("Failed to insert subscriber.")?;

                queries::store_token(conn, &subscription_token, &subscriber_id)
                    .await
                    .context("Failed to store token.")?;
                send_confirmation_email(
                    &app_state.email_client,
                    new_subscriber,
                    &app_state.base_url,
                    &subscription_token,
                )
                .await
                .context("Failed to send confirmation email.")?;
                Ok(())
            }
            .scope_boxed()
        })
        .await?;
    Ok(StatusCode::OK)
}

#[derive(thiserror::Error, Debug)]
pub enum SubscriptionError {
    #[error("{0}")]
    InvalidSubscriberData(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("Unkown database error.")]
    DatabaseError(#[from] diesel::result::Error),
}

impl IntoResponse for SubscriptionError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct SubscriberErrorResponse {
            message: String,
        }
        tracing::error!("{} Reason: {:?}", self, self);
        let (status, message) = match self {
            SubscriptionError::InvalidSubscriberData(message) => {
                (StatusCode::BAD_REQUEST, message)
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong".to_string(),
            ),
        };
        (status, axum::Json(SubscriberErrorResponse { message }))
            .into_response()
    }
}
