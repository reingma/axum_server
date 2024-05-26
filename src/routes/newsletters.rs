use anyhow::Context;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::{
    database::queries::get_confirmed_subscribers, startup::ApplicationState,
};

pub async fn publish_newsletter(
    State(app_state): State<ApplicationState>,
    body: Json<BodyData>,
) -> Result<StatusCode, PublishNewsletterError> {
    let mut connection =
        crate::database::get_connection(app_state.database_pool).await;
    let subscribers = get_confirmed_subscribers(&mut connection)
        .await
        .context("Could not get confirmed subscribers")?;
    let email_client = app_state.email_client;
    for subscriber in subscribers {
        match subscriber {
            Ok(valid_subscriber) => {
                email_client
                    .send_email(
                        &valid_subscriber.confirmed_email,
                        &body.content.text,
                        &body.content.html,
                        &body.title,
                    )
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to send newsletter issue to {}",
                            valid_subscriber.confirmed_email
                        )
                    })?;
            }
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber. \
                     Their stored contact details are invalid."
                );
            }
        }
    }
    Ok(StatusCode::OK)
}

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

#[derive(thiserror::Error, Debug)]
pub enum PublishNewsletterError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for PublishNewsletterError {
    fn into_response(self) -> axum::response::Response {
        #[derive(serde::Serialize)]
        struct PublishNewsletterErrorResponse {
            message: String,
        }
        tracing::error!("{} Reason {:?}", self, self);
        let (status, message) = match self {
            PublishNewsletterError::UnexpectedError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong".to_string(),
            ),
        };
        (
            status,
            axum::Json(PublishNewsletterErrorResponse { message }),
        )
            .into_response()
    }
}
