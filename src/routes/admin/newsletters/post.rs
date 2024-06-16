use crate::{
    authentication::UserId, database::queries::get_confirmed_subscribers,
    startup::ApplicationState, utils::redirect_with_flash,
};
use anyhow::{anyhow, Context};
use axum::{
    extract::State,
    http::{Response, StatusCode},
    response::{IntoResponse, Redirect},
    Extension, Form,
};
use axum_extra::extract::SignedCookieJar;

#[derive(serde::Deserialize)]
pub struct NewsletterForm {
    title: String,
    content_text: String,
    content_html: String,
}
#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(app_state, form),
    fields( user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    State(app_state): State<ApplicationState>,
    jar: SignedCookieJar,
    Extension(valid_id): Extension<UserId>,
    Form(form): Form<NewsletterForm>,
) -> Result<(SignedCookieJar, Redirect), PublishNewsletterError> {
    let mut connection =
        crate::database::get_connection(app_state.database_pool)
            .await
            .context("Could not get database pool")?;
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
                        &form.content_text,
                        &form.content_html,
                        &form.title,
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
    Ok(redirect_with_flash(
        "/admin/newsletters",
        anyhow!("Newsletter delivered successfully"),
        jar,
    ))
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
