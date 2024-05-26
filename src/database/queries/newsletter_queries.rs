use crate::database::DatabaseConnection;
use crate::domain::SubscriberEmail;
use crate::schema::subscriptions::dsl::*;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

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
