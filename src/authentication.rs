use anyhow::Context;
use argon2::{
    password_hash::SaltString, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier,
};
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use secrecy::{ExposeSecret, Secret};
use uuid::Uuid;

use crate::{
    database::{
        queries::{
            change_password_query, get_stored_credentials, ValidateUserError,
        },
        DatabaseConnection,
    },
    domain::Password,
    session_state::TypedSession,
    telemetry::spawn_blocking_with_tracing,
};

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub struct Credentials {
    pub username: String,
    pub password: Password,
}

#[tracing::instrument(
    name = "Validate Credentials",
    skip(connection, credentials)
)]
pub async fn validate_credentials(
    credentials: Credentials,
    connection: &mut DatabaseConnection,
) -> Result<uuid::Uuid, AuthError> {
    let (stored_user_id, expected_hash) =
        match get_stored_credentials(&credentials.username, connection).await {
            Ok(row) => (Some(row.0), row.1),
            Err(e) => match e {
                ValidateUserError::DatabaseError(e) => {
                    return Err(AuthError::UnexpectedError(e.into()))
                }
                ValidateUserError::AuthenticationError(_) => (
                    None,
                    Secret::new(
                        "$argon2id$v=19$m=15000,t=2,p=1$\
                    gZiV/M1gPc22ElAH/Jh1Hw$\
                    CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
                            .to_string(),
                    ),
                ),
            },
        };

    crate::telemetry::spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_hash, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task.")
    .map_err(AuthError::InvalidCredentials)??;

    stored_user_id.ok_or_else(|| {
        AuthError::InvalidCredentials(anyhow::anyhow!("Invalid username."))
    })
}

#[tracing::instrument(
    name = "Verify password hash",
    skip(expected_password_hash, password_candidate)
)]
fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Password,
) -> Result<(), AuthError> {
    let expected_hash =
        PasswordHash::new(expected_password_hash.expose_secret())
            .context("Failed to parse hash in PHC format.")
            .map_err(AuthError::UnexpectedError)?;
    Argon2::default()
        .verify_password(
            password_candidate.as_ref().expose_secret().as_bytes(),
            &expected_hash,
        )
        .context("Invalid password.")
        .map_err(AuthError::InvalidCredentials)?;
    Ok(())
}

#[tracing::instrument(
    name = "Middleware Credential Checking",
    skip(session, request, next)
)]
pub async fn check_credentials(
    session: TypedSession,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, Response> {
    if session
        .get_user_id()
        .await
        .context("Could not confirm user login")
        .map_err(|err| {
            (StatusCode::UNAUTHORIZED, err.to_string()).into_response()
        })?
        .is_none()
    {
        Ok(Redirect::to("/login").into_response())
    } else {
        Ok(next.run(request).await)
    }
}

#[tracing::instrument(name = "Change password", skip(password, connection))]
pub async fn change_password(
    user_id: Uuid,
    password: Password,
    connection: &mut DatabaseConnection,
) -> Result<(), anyhow::Error> {
    let password_hash =
        spawn_blocking_with_tracing(move || compute_password_hash(&password))
            .await?
            .context("Failed to hash password.")?;
    change_password_query(connection, user_id, password_hash)
        .await
        .context("Failed to change password in the database.")?;
    Ok(())
}

fn compute_password_hash(
    password: &Password,
) -> Result<Secret<String>, anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    )
    .hash_password(password.as_ref().expose_secret().as_bytes(), &salt)?
    .to_string();
    Ok(Secret::new(password_hash))
}
