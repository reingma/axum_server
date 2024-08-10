use crate::{
    authentication::UserId, database::queries::get_confirmed_subscribers,
    idempotency::IdempotencyKey, startup::ApplicationState,
    utils::redirect_with_flash,
};
use anyhow::{anyhow, Context};
use axum::{
    async_trait,
    extract::{
        rejection::FormRejection, FromRef, FromRequest, FromRequestParts,
        Request, State,
    },
    http::{Response, StatusCode},
    response::{IntoResponse, Redirect},
    Extension,
};
use axum_extra::extract::SignedCookieJar;
use cookie::Key;

#[derive(serde::Deserialize)]
pub struct NewsletterForm {
    title: String,
    content_text: String,
    content_html: String,
    idempotency_key: String,
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
    let NewsletterForm {
        title,
        content_text,
        content_html,
        idempotency_key,
    } = form;
    let idempotency_key: IdempotencyKey = idempotency_key
        .try_into()
        .context("Failed to parse idempotency key")?;
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
                        &content_text,
                        &content_html,
                        &title,
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
        //TODO: there has to be a better way to handle errors.
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

pub struct Form<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for Form<T>
where
    axum::Form<T>: FromRequest<S, Rejection = FormRejection>,
    S: Send + Sync,
    Key: FromRef<S>,
{
    type Rejection = (SignedCookieJar, Redirect);

    async fn from_request(
        req: Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let jar = SignedCookieJar::from_request_parts(&mut parts, state)
            .await
            .expect("Failed to extract the cookie jar.");
        let req = Request::from_parts(parts, body);

        match axum::Form::<T>::from_request(req, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(_) => Err(redirect_with_flash(
                "/admin/newsletters",
                anyhow!("Invalid newsletter body."),
                jar,
            )),
        }
    }
}
