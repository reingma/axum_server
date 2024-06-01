use anyhow::Context;
use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
};
use tracing::instrument;

use crate::TEMPLATES;

#[instrument(name = "Requested landing page")]
pub async fn home() -> Result<Response<String>, HomePageError> {
    let tera_context = tera::Context::new();
    let html_body = TEMPLATES
        .render("pages/home.html", &tera_context)
        .context("Could not render home page.")?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "text/html")
        .body(html_body)
        .context("Could not create response.")?)
}

#[derive(thiserror::Error, Debug)]
pub enum HomePageError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for HomePageError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{} Reason {:?}", self, self);
        match self {
            HomePageError::UnexpectedError(_) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("Something went wrong.".into())
                .unwrap(),
        }
    }
}
