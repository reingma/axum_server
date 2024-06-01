use anyhow::Context;
use axum::{
    extract::State,
    http::{Response, StatusCode},
    response::{IntoResponse, Redirect},
    Form,
};
use hmac::Mac;
use secrecy::{ExposeSecret, Secret};
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
#[instrument(skip(app_state,form), fields(username=tracing::field::Empty, user_id=tracing::field::Empty))]
pub async fn login(
    State(app_state): State<ApplicationState>,
    Form(form): Form<FormData>,
) -> Result<Redirect, LoginError> {
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
            Ok(Redirect::to("/"))
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
            let query_string =
                format!("error={}", urlencoding::Encoded::new(e.to_string()));
            let hmac_tag = {
                let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(
                    app_state.secret.0.expose_secret().as_bytes(),
                )
                .unwrap();
                mac.update(query_string.as_bytes());
                mac.finalize().into_bytes()
            };
            Ok(Redirect::to(&format!(
                "/login?{}&tag={:x}",
                query_string, hmac_tag
            )))
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
