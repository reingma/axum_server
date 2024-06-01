use crate::database::queries::ValidateUserError;
use crate::database::DatabaseConnection;
use crate::{
    database::queries::{get_confirmed_subscribers, get_stored_credentials},
    startup::ApplicationState,
};
use anyhow::Context;
use argon2::Argon2;
use argon2::PasswordHash;
use argon2::PasswordVerifier;
use axum::{
    extract::State,
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use base64::Engine;
use secrecy::ExposeSecret;
use secrecy::Secret;

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
    let valid_id = validate_credentials(credentials, &mut connection).await?;
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

#[tracing::instrument(
    name = "Validate Credentials",
    skip(connection, credentials)
)]
pub async fn validate_credentials(
    credentials: Credentials,
    connection: &mut DatabaseConnection,
) -> Result<uuid::Uuid, PublishNewsletterError> {
    let (stored_user_id, expected_hash) =
        match get_stored_credentials(&credentials.username, connection).await {
            Ok(row) => (Some(row.0), row.1),
            Err(e) => match e {
                ValidateUserError::DatabaseError(e) => {
                    return Err(PublishNewsletterError::UnexpectedError(
                        e.into(),
                    ))
                }
                ValidateUserError::AuthenticationError(_) => (
                    None,
                    Secret::new(
                        "$argon2id$v=19$m=15000,t=2,p=1$\
                    gZiV/M1gPc22ElAH/Jh1Hw$\
                    CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
                            .to_string(),
                    ),
                ),
            },
        };

    crate::telemetry::spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_hash, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task.")
    .map_err(PublishNewsletterError::UnexpectedError)??;

    stored_user_id.ok_or_else(|| {
        PublishNewsletterError::AuthError(anyhow::anyhow!("Invalid username."))
    })
}

#[tracing::instrument(
    name = "Verify password hash",
    skip(expected_password_hash, password_candidate)
)]
fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> Result<(), PublishNewsletterError> {
    let expected_hash =
        PasswordHash::new(expected_password_hash.expose_secret())
            .context("Failed to parse hash in PHC format.")
            .map_err(PublishNewsletterError::UnexpectedError)?;
    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_hash,
        )
        .context("Invalid password.")
        .map_err(PublishNewsletterError::AuthError)?;
    Ok(())
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
