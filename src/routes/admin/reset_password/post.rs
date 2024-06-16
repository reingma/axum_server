use anyhow::{anyhow, Context};
use axum::{
    extract::State,
    http::{Response, StatusCode},
    response::{IntoResponse, Redirect},
    Extension, Form,
};
use axum_extra::extract::SignedCookieJar;
use secrecy::{ExposeSecret, Secret};
use tracing::instrument;

use crate::{
    authentication::{validate_credentials, AuthError, Credentials, UserId},
    database::queries::get_username,
    domain::Password,
    startup::ApplicationState,
    utils::redirect_with_flash,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}
#[instrument(skip(app_state, form, jar))]
pub async fn change_pasword(
    State(app_state): State<ApplicationState>,
    jar: SignedCookieJar,
    Extension(user_id): Extension<UserId>,
    Form(form): Form<FormData>,
) -> Result<(SignedCookieJar, Redirect), PasswordResetError> {
    let mut connection =
        crate::database::get_connection(app_state.database_pool)
            .await
            .context("Failed to get database pool")?;
    let new_pass =
        match Password::try_from(form.new_password.expose_secret().to_string())
        {
            Ok(pass) => pass,
            Err(err) => {
                return Ok(redirect_with_flash(
                    "/admin/password",
                    anyhow!(err),
                    jar,
                ))
            }
        };
    if form.new_password_check.expose_secret()
        != form.new_password.expose_secret()
    {
        return Ok(redirect_with_flash(
            "/admin/password",
            anyhow!(
                "You entered two different new passwords - \
                the field values must match."
            ),
            jar,
        ));
    }
    let username = get_username(&mut connection, *user_id)
        .await
        .context("Failed to get user information.")?;
    let password = match Password::try_from(
        form.current_password.expose_secret().to_string(),
    ) {
        Ok(pass) => pass,
        Err(_) => {
            return Ok(redirect_with_flash(
                "/admin/password",
                anyhow!("The current password is incorrect."),
                jar,
            ));
        }
    };
    let credentials = Credentials { username, password };
    if let Err(e) = validate_credentials(credentials, &mut connection).await {
        return match e {
            AuthError::UnexpectedError(_) => {
                Err(PasswordResetError::AuthError(e.into()))
            }
            AuthError::InvalidCredentials(_) => Ok(redirect_with_flash(
                "/admin/password",
                anyhow!("The current password is incorrect."),
                jar,
            )),
        };
    } else {
        crate::authentication::change_password(
            *user_id,
            new_pass,
            &mut connection,
        )
        .await?;
        Ok(redirect_with_flash(
            "/admin/password",
            anyhow!("Your password has been changed."),
            jar,
        ))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PasswordResetError {
    #[error("Something went wrong.")]
    UnexpectedError(#[from] anyhow::Error),
    #[error("Authentication failed.")]
    AuthError(#[source] anyhow::Error),
}

impl IntoResponse for PasswordResetError {
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
