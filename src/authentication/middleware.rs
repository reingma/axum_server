use std::{fmt::Display, ops::Deref};

use crate::session_state::TypedSession;
use anyhow::Context;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use uuid::Uuid;

#[tracing::instrument(
    name = "Middleware Credential Checking",
    skip(session, request, next)
)]
pub async fn check_credentials(
    session: TypedSession,
    mut request: Request,
    next: Next,
) -> Result<impl IntoResponse, Response> {
    let id = match session
        .get_user_id()
        .await
        .context("Could not confirm user login")
        .map_err(|err| {
            (StatusCode::UNAUTHORIZED, err.to_string()).into_response()
        })? {
        Some(id) => id,
        None => {
            return Ok(Redirect::to("/login").into_response());
        }
    };

    request.extensions_mut().insert(UserId(id));
    Ok(next.run(request).await)
}

#[derive(Debug, Clone, Copy)]
pub struct UserId(Uuid);

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for UserId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
