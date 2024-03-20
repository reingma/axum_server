use crate::{
    database::DatabaseConnection,
    models::SubscriptionTokens,
    schema::{self, subscriptions},
};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel::result::Error;
use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;
use schema::subscription_tokens::dsl::*;
use schema::subscriptions::dsl::*;
use uuid::Uuid;

#[tracing::instrument(
    name = "Storing subscriber token into database",
    skip(subscriber_token, connection)
)]
pub async fn store_token(
    connection: &mut DatabaseConnection,
    subscriber_token: &str,
    sub_id: &Uuid,
) -> Result<(), Error> {
    let token_entry = SubscriptionTokens::new(subscriber_token, sub_id);
    match diesel::insert_into(schema::subscription_tokens::table)
        .values(&token_entry)
        .execute(connection)
        .await
    {
        Ok(_) => {
            tracing::info!("New subscriber details have been saved");
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to execute query {:?}", e);
            return Err(e);
        }
    }
}

#[tracing::instrument(
    name = "Getting the subscriber id for the token",
    skip(token, connection)
)]
pub async fn get_subscriber_id_for_token(
    connection: &mut DatabaseConnection,
    token: &str,
) -> Result<Option<Uuid>, diesel::result::Error> {
    let subscriber = subscription_tokens
        .filter(
            subscription_token
                .eq(token)
                .and(generated_at.gt(Utc::now() - Duration::days(1))),
        )
        .select(SubscriptionTokens::as_select())
        .first(connection)
        .await
        .optional()
        .map_err(|e| {
            tracing::error!("Failed to execute query {:?}", e);
            e
        })?;
    let value = match subscriber {
        Some(sub) => Some(sub.subscriber_id),
        None => None,
    };
    Ok(value)
}

#[tracing::instrument(name = "Set subscriber to confirmed", skip(connection))]
pub async fn confirm_subscriber(
    connection: &mut DatabaseConnection,
    sub_id: &Uuid,
) -> Result<(), diesel::result::Error> {
    match diesel::update(subscriptions::table.find(sub_id))
        .set(status.eq("confirmed"))
        .execute(connection)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::error!("Failed to execute query {:?}", e);
            Err(e)
        }
    }
}
