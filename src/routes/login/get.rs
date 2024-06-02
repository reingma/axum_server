use anyhow::Context;
use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
};
use axum_extra::extract::SignedCookieJar;
use cookie::Cookie;
use tracing::instrument;

use crate::TEMPLATES;

#[instrument(name = "Requesting login page")]
pub async fn login_form(
    jar: SignedCookieJar,
) -> Result<(SignedCookieJar, Response<String>), LoginFormError> {
    let mut tera_context = tera::Context::new();
    let error_html = if let Some(error) =
        jar.get("_flash").map(|cookie| cookie.value().to_owned())
    {
        error.to_string()
    } else {
        "".to_string()
    };
    tera_context.insert("error", &error_html);
    let html_body = TEMPLATES
        .render("pages/login.html", &tera_context)
        .context("Could not render login page.")?;
    let jar = jar.remove(Cookie::from("_flash"));
    Ok((
        jar,
        Response::builder()
            .status(StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, "text/html")
            .body(html_body)
            .context("Could not create response.")?,
    ))
}

#[derive(thiserror::Error, Debug)]
pub enum LoginFormError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for LoginFormError {
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
