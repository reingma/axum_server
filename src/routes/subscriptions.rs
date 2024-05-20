use std::sync::Arc;

use crate::domain::SubscriptionToken;
use crate::{
    database::queries, database::queries::insert_subscriber,
    domain::NewSubscriber, email_client::send::send_confirmation_email,
    startup::ApplicationState,
};
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
    let new_subscriber: NewSubscriber = match subscriber.try_into() {
        Ok(new_subscriber) => new_subscriber,
        Err(e) => return Err(SubscriptionError::InvalidSubscriberData(e)),
    };
    tracing::info!("Adding a new subscriber to the database.");

    let mut connection =
        crate::database::get_connection(app_state.database_pool).await;

    let subscription_token = Arc::new(SubscriptionToken::generate());
    match connection
        .transaction::<_, diesel::result::Error, _>(|conn| {
            async move {
                let subscriber_id =
                    insert_subscriber(conn, &new_subscriber).await?;

                queries::store_token(conn, &subscription_token, &subscriber_id)
                    .await?;
                match send_confirmation_email(
                    &app_state.email_client,
                    new_subscriber,
                    &app_state.base_url,
                    &subscription_token,
                )
                .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        tracing::error!(
                            "Subscription transaction failed. Could not send Email with reason {:?}",
                            e
                        );
                        Err(diesel::result::Error::RollbackTransaction)
                    }
                }
            }
            .scope_boxed()
        })
        .await
    {
        Ok(()) => return Ok(StatusCode::OK),
        Err(e) => match e {
            diesel::result::Error::RollbackTransaction => {
                return Err(SubscriptionError::ConfirmationEmailError(
                    "Could not send Email.".to_string(),
                ))
            }
            _ => return Err(e.into()),
        },
    }
}

pub enum SubscriptionError {
    InsertSubscriberError(diesel::result::Error),
    InvalidSubscriberData(String),
    ConfirmationEmailError(String),
}

impl IntoResponse for SubscriptionError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct SubscriberErrorResponse {
            message: String,
        }
        let (status, message) = match self {
            SubscriptionError::InvalidSubscriberData(message) => {
                (StatusCode::BAD_REQUEST, message)
            }
            SubscriptionError::ConfirmationEmailError(message) => {
                (StatusCode::INTERNAL_SERVER_ERROR, message)
            }
            SubscriptionError::InsertSubscriberError(_e) => {
                tracing::error!(
                    "Failed when storing subscriber data into the database."
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_string(),
                )
            }
        };
        (status, axum::Json(SubscriberErrorResponse { message }))
            .into_response()
    }
}

impl From<diesel::result::Error> for SubscriptionError {
    fn from(value: diesel::result::Error) -> Self {
        Self::InsertSubscriberError(value)
    }
}
