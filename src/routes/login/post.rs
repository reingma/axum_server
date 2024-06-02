use anyhow::Context;
use axum::{
    extract::State,
    http::{Response, StatusCode},
    response::{IntoResponse, Redirect},
    Form,
};
use axum_extra::extract::{cookie::Cookie, SignedCookieJar};
use secrecy::Secret;
use tracing::instrument;

use crate::{
    authentication::{validate_credentials, AuthError, Credentials},
    startup::ApplicationState,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}
#[instrument(skip(app_state,form,jar), fields(username=tracing::field::Empty, user_id=tracing::field::Empty))]
pub async fn login(
    State(app_state): State<ApplicationState>,
    jar: SignedCookieJar,
    Form(form): Form<FormData>,
) -> Result<(SignedCookieJar, Redirect), LoginError> {
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };
    tracing::Span::current()
        .record("username", &tracing::field::display(&credentials.username));
    let mut connection =
        crate::database::get_connection(app_state.database_pool)
            .await
            .context("Could not get database pool")?;
    match validate_credentials(credentials, &mut connection).await {
        Ok(user_id) => {
            tracing::Span::current()
                .record("user_id", &tracing::field::display(&user_id));
            Ok((jar, Redirect::to("/login")))
        }
        Err(e) => {
            let e = match e {
                AuthError::UnexpectedError(_) => {
                    return Err(LoginError::UnexpectedError(e.into()))
                }
                AuthError::InvalidCredentials(_) => {
                    LoginError::AuthError(e.into())
                }
            };
            tracing::error!("{} Reason {:?}", e, e);
            let cookie = Cookie::new("_flash", e.to_string());
            Ok((jar.add(cookie), Redirect::to("/login")))
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LoginError {
    #[error("Something went wrong.")]
    UnexpectedError(#[from] anyhow::Error),
    #[error("Authentication failed.")]
    AuthError(#[source] anyhow::Error),
}

impl IntoResponse for LoginError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{} Reason {:?}", self, self);
        match self {
            Self::UnexpectedError(_) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Something went wrong.".into())
                .unwrap(),
            Self::AuthError(_) => Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body("Authentication failed.".into())
                .unwrap(),
        }
    }
}
