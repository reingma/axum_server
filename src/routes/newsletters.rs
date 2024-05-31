use anyhow::Context;
use axum::{
    extract::State,
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use base64::Engine;
use secrecy::Secret;

use crate::{
    database::queries::{
        get_confirmed_subscribers, validate_credentials, ValidateUserError,
    },
    startup::ApplicationState,
};

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(app_state, body, headers),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    State(app_state): State<ApplicationState>,
    headers: HeaderMap,
    body: Json<BodyData>,
) -> Result<StatusCode, PublishNewsletterError> {
    let mut connection =
        crate::database::get_connection(app_state.database_pool).await;
    let credentials = basic_authentication(headers)
        .map_err(PublishNewsletterError::AuthError)?;
    tracing::Span::current()
        .record("username", &tracing::field::display(&credentials.username));
    let valid_id = match validate_credentials(&credentials, &mut connection)
        .await
    {
        Ok(valid_id) => valid_id,
        Err(error) => match error {
            ValidateUserError::DatabaseError(e) => {
                return Err(PublishNewsletterError::UnexpectedError(e.into()))
            }
            ValidateUserError::AuthenticationError(message) => {
                return Err(PublishNewsletterError::AuthError(anyhow::anyhow!(
                    message
                )))
            }
        },
    };
    tracing::Span::current()
        .record("user_id", &tracing::field::display(&valid_id));
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
    tracing::info!("Email delivered to subscribers.");
    Ok(StatusCode::OK)
}
pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

fn basic_authentication(
    headers: HeaderMap,
) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header was missing")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string")?;
    let base64segment = header_value
        .strip_prefix("Basic ")
        .context("Scheme was not 'Basic'.")?;
    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64segment)
        .context("Failed to base64-decode 'Basic' credentials")?;
    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid utf8")?;
    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| {
            anyhow::anyhow!("A username must be provided in 'Basic' auth.")
        })?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| {
            anyhow::anyhow!("A password must be provided in 'Basic' auth.")
        })?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
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
    #[error("Authentication Failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for PublishNewsletterError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{} Reason {:?}", self, self);
        match self {
            PublishNewsletterError::UnexpectedError(_) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Something went wrong.".into())
                .unwrap(),
            PublishNewsletterError::AuthError(_) => Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header(
                    axum::http::header::WWW_AUTHENTICATE,
                    r#"Basic realm="publish""#,
                )
                .body("Unauthorized Access".into())
                .unwrap(),
        }
    }
}
