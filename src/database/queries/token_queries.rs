use crate::{
    database::DatabaseConnection,
    domain::SubscriptionToken,
    models::SubscriptionTokens,
    schema::{self, subscriptions},
};
use chrono::{Duration, Utc};
use diesel::prelude::*;
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
    subscriber_token: &SubscriptionToken,
    sub_id: &Uuid,
) -> Result<(), StoreTokenError> {
    let token_entry =
        SubscriptionTokens::new(subscriber_token.as_ref(), sub_id);
    diesel::insert_into(schema::subscription_tokens::table)
        .values(&token_entry)
        .execute(connection)
        .await?;
    tracing::info!("New subscriber details have been saved");
    Ok(())
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
        .optional()?;
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
    diesel::update(subscriptions::table.find(sub_id))
        .set(status.eq("confirmed"))
        .execute(connection)
        .await?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
#[error("A database error has ocurred when storing a subscription token")]
pub struct StoreTokenError(#[from] diesel::result::Error);
