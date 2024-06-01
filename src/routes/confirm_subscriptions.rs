use anyhow::Context;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;

use crate::database::{
    queries::{confirm_subscriber, get_subscriber_id_for_token},
    DatabaseConnectionPool,
};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(
    name = "Confirming Pending Subscriber",
    skip(parameters, database_pool)
)]
pub async fn confirm(
    parameters: Query<Parameters>,
    State(database_pool): State<DatabaseConnectionPool>,
) -> Result<StatusCode, ConfirmationError> {
    let mut connection = crate::database::get_connection(database_pool)
        .await
        .context("Failed to get database pool.")?;
    let subscriber_id = get_subscriber_id_for_token(
        &mut connection,
        &parameters.subscription_token,
    )
    .await
    .context("Failed to find subscriber.")?;
    match subscriber_id {
        None => Err(ConfirmationError::InvalidToken(
            "Invalid token.".to_string(),
        )),
        Some(id) => {
            confirm_subscriber(&mut connection, &id)
                .await
                .context("Could not confirm subscriber.")?;
            Ok(StatusCode::OK)
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ConfirmationError {
    #[error("{0}")]
    InvalidToken(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for ConfirmationError {
    fn into_response(self) -> axum::response::Response {
        #[derive(Serialize)]
        struct ConfirmResponse {
            message: String,
        }
        tracing::error!("{} Reason: {:?}", self, self);
        let (status, message) = match self {
            Self::InvalidToken(error_message) => {
                (StatusCode::UNAUTHORIZED, error_message)
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong".to_string(),
            ),
        };
        (status, axum::Json(ConfirmResponse { message })).into_response()
    }
}
