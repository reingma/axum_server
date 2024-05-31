use crate::database::DatabaseConnection;
use crate::domain::SubscriberEmail;
use crate::routes::Credentials;
use crate::schema::subscriptions::dsl::*;
use crate::schema::users::dsl::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use secrecy::ExposeSecret;
use sha3::Digest;

pub struct ConfirmedSubscriber {
    pub confirmed_email: SubscriberEmail,
}

#[tracing::instrument(name = "Get Confirmed subscribers", skip(connection))]
pub async fn get_confirmed_subscribers(
    connection: &mut DatabaseConnection,
) -> Result<Vec<Result<ConfirmedSubscriber, String>>, diesel::result::Error> {
    let emails: Vec<String> = subscriptions
        .filter(status.eq("confirmed"))
        .select(email)
        .load(connection)
        .await?;
    let confirmed_subscribers = emails
        .iter()
        .map(|confirmed_email| {
            match SubscriberEmail::try_from(confirmed_email.to_string()) {
                Ok(valid) => Ok(ConfirmedSubscriber {
                    confirmed_email: valid,
                }),
                Err(error) => Err(error),
            }
        })
        .collect();
    Ok(confirmed_subscribers)
}

pub async fn validate_credentials(
    credentials: &Credentials,
    connection: &mut DatabaseConnection,
) -> Result<uuid::Uuid, ValidateUserError> {
    let hash =
        sha3::Sha3_256::digest(credentials.password.expose_secret().as_bytes());
    let hash = format!("{:x}", hash);
    let validated_id: Option<uuid::Uuid> = users
        .filter(
            username
                .eq(&credentials.username)
                .and(password_hash.eq(hash)),
        )
        .select(user_id)
        .first(connection)
        .await
        .optional()?;

    validated_id.ok_or_else(|| {
        ValidateUserError::AuthenticationError(
            "Invalid username or password".into(),
        )
    })
}

#[derive(Debug, thiserror::Error)]
pub enum ValidateUserError {
    #[error("Could not fetch user data.")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("Invalid username or password.")]
    AuthenticationError(String),
}
