use anyhow::Context;
use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use axum_extra::extract::SignedCookieJar;
use tracing::instrument;

use crate::{utils::get_flash_error, TEMPLATES};

#[instrument(name = "Requesting newsletters page", skip(jar))]
pub async fn newsletters_form(
    jar: SignedCookieJar,
) -> Result<(SignedCookieJar, Response<Body>), NewsletterFormError> {
    let mut tera_context = tera::Context::new();
    let (jar, message) = get_flash_error(jar);
    tera_context.insert("message", &message);
    tera_context.insert("idempotency_key", &uuid::Uuid::now_v7());
    let html_body = TEMPLATES
        .render("pages/newsletters.html", &tera_context)
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
pub enum NewsletterFormError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for NewsletterFormError {
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
