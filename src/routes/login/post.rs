use crate::{domain::Password, utils::redirect_with_flash};
use anyhow::{anyhow, Context};
use axum::{
    extract::State,
    http::{Response, StatusCode},
    response::{IntoResponse, Redirect},
    Form,
};
use axum_extra::extract::SignedCookieJar;
use secrecy::{ExposeSecret, Secret};
use tracing::instrument;

use crate::{
    authentication::{validate_credentials, AuthError, Credentials},
    session_state::TypedSession,
    startup::ApplicationState,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}
#[instrument(skip(app_state,form,jar, session), fields(username=tracing::field::Empty, user_id=tracing::field::Empty))]
pub async fn login(
    State(app_state): State<ApplicationState>,
    session: TypedSession,
    jar: SignedCookieJar,
    Form(form): Form<FormData>,
) -> Result<(SignedCookieJar, Redirect), LoginError> {
    let password =
        match Password::try_from(form.password.expose_secret().to_string()) {
            Ok(pass) => pass,
            Err(_) => {
                return Ok(redirect_with_flash(
                    "/login",
                    anyhow!("Invalid password."),
                    jar,
                ))
            }
        };
    let credentials = Credentials {
        username: form.username,
        password,
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
            session.cycle_id().await.context("Session failure")?;
            if let Err(e) = session
                .insert_user_id(user_id)
                .await
                .context("Could not store user_id")
            {
                return Ok(redirect_with_flash("/login", e, jar));
            };
            Ok((jar, Redirect::to("/admin/dashboard")))
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
            Ok(redirect_with_flash("/login", e.into(), jar))
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
