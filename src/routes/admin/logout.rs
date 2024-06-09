use anyhow::{anyhow, Context};
use axum::{
    http::{Response, StatusCode},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::SignedCookieJar;
use tracing::instrument;

use crate::{session_state::TypedSession, utils::redirect_with_flash};

#[instrument(skip(jar, session))]
pub async fn logout(
    jar: SignedCookieJar,
    session: TypedSession,
) -> Result<(SignedCookieJar, Redirect), LogoutError> {
    if session
        .get_user_id()
        .await
        .context("Faild to get session data.")?
        .is_none()
    {
        Ok((jar, Redirect::to("/admin/dashboard")))
    } else {
        session.logout().await.context("Failed to logout.")?;
        Ok(redirect_with_flash(
            "/login",
            anyhow!("You have successfully logged out."),
            jar,
        ))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LogoutError {
    #[error("Something went wrong.")]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for LogoutError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{} Reason {:?}", self, self);
        match self {
            Self::UnexpectedError(_) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Something went wrong.".into())
                .unwrap(),
        }
    }
}
