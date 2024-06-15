use anyhow::Context;
use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use axum_extra::extract::SignedCookieJar;
use tracing::instrument;

use crate::{utils::get_flash_error, TEMPLATES};

#[instrument(name = "Requesting reset_password page", skip(jar))]
pub async fn reset_password_form(
    jar: SignedCookieJar,
) -> Result<(SignedCookieJar, Response<Body>), PasswordFormError> {
    let (jar, html_error) = get_flash_error(jar);
    let mut tera_context = tera::Context::new();
    tera_context.insert("error", &html_error);
    let html_body = TEMPLATES
        .render("pages/reset_password.html", &tera_context)
        .context("Could not render login page.")?;
    Ok((
        jar,
        Response::builder()
            .status(StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, "text/html")
            .body(html_body.into())
            .context("Could not create response.")?,
    ))
}

#[derive(thiserror::Error, Debug)]
pub enum PasswordFormError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for PasswordFormError {
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
