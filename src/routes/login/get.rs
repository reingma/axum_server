use anyhow::Context;
use axum::{
    extract::{Query, State},
    http::{Response, StatusCode},
    response::IntoResponse,
};
use hmac::Mac;
use secrecy::ExposeSecret;
use tracing::instrument;

use crate::{
    startup::{ApplicationState, HmacSecret},
    TEMPLATES,
};

#[derive(serde::Deserialize)]
pub struct LoginQueryParameters {
    error: String,
    tag: String,
}

impl LoginQueryParameters {
    fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag)?;
        let query_string =
            format!("error={}", urlencoding::Encoded::new(&self.error));
        let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(
            secret.0.expose_secret().as_bytes(),
        )
        .unwrap();
        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)?;
        Ok(self.error)
    }
}

#[instrument(name = "Requesting login page", skip(app_state, query_params))]
pub async fn login_form(
    State(app_state): State<ApplicationState>,
    query_params: Option<Query<LoginQueryParameters>>,
) -> Result<Response<String>, LoginFormError> {
    let error_html = match query_params {
        Some(params) => match params.0.verify(&app_state.secret) {
            Ok(error) => error,
            Err(e) => {
                tracing::warn!(error.message = %e, error.cause_chain = %e, "Failed to verify query parameters using HMAC tag");
                "".into()
            }
        },
        None => "".into(),
    };
    let mut tera_context = tera::Context::new();
    tera_context.insert("error", &error_html);
    let html_body = TEMPLATES
        .render("pages/login.html", &tera_context)
        .context("Could not render login page.")?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "text/html")
        .body(html_body)
        .context("Could not create response.")?)
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
