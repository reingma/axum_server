use crate::database::DatabaseConnection;
use crate::domain::NewSubscriber;
use crate::{models::Subscriptions, schema::subscriptions};
use diesel::result::Error;
use diesel_async::RunQueryDsl;

#[tracing::instrument(
    name = "Inserting subscriber into databse.",
    skip(subscriber_data, connection)
)]
pub async fn insert_subscriber(
    subscriber_data: NewSubscriber,
    connection: &mut DatabaseConnection,
) -> Result<(), Error> {
    let subscription_entry = Subscriptions::new(
        subscriber_data.email,
        subscriber_data.name.inner_ref().to_string(),
    );
    match diesel::insert_into(subscriptions::table)
        .values(subscription_entry)
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
