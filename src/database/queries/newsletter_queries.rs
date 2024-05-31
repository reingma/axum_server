use crate::database::DatabaseConnection;
use crate::domain::SubscriberEmail;
use crate::schema::subscriptions::dsl::*;
use crate::schema::users::dsl::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use secrecy::Secret;

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

#[tracing::instrument(
    name = "Retrieve stored credentials",
    skip(uname, connection)
)]
pub async fn get_stored_credentials(
    uname: &str,
    connection: &mut DatabaseConnection,
) -> Result<(uuid::Uuid, Secret<String>), ValidateUserError> {
    let row: Option<(uuid::Uuid, String)> = users
        .filter(username.eq(&uname))
        .select((user_id, password_hash))
        .first(connection)
        .await
        .optional()?;
    Ok(match row {
        Some(row) => (row.0, Secret::new(row.1)),
        None => {
            return Err(ValidateUserError::AuthenticationError(
                "Unknown username".into(),
            ));
        }
    })
}

#[derive(Debug, thiserror::Error)]
pub enum ValidateUserError {
    #[error("Could not fetch user data.")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("Invalid username or password.")]
    AuthenticationError(String),
}
